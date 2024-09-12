/*****************************************************************************
 *   Ledger App Ironfish Rust.
 *   (c) 2023 Ledger SAS.
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 *****************************************************************************/

use crate::{AppSW, Instruction};
use alloc::vec::Vec;
use ironfish_frost::dkg::group_key::{GroupSecretKey, GROUP_SECRET_KEY_LEN};
use ironfish_frost::frost::keys::PublicKeyPackage as FrostPublicKeyPackage;
use ironfish_frost::dkg::round3::PublicKeyPackage;
use ledger_device_sdk::io::{Comm, Event};
use crate::ironfish::multisig::{derive_account_keys, MultisigAccountKeys};
use crate::utils::{zlog_stack};
use crate::nvm::dkg_keys::DkgKeys;

const MAX_APDU_SIZE: usize = 253;

#[inline(never)]
pub fn handler_dkg_get_keys(
    comm: &mut Comm,
    key_type: &u8
) -> Result<(), AppSW> {
    zlog_stack("start handler_dkg_get_keys\0");

    let group_secret_key = DkgKeys.load_group_secret_key()?;
    let frost_public_key_package = DkgKeys.load_frost_public_key_package()?;

    let verifying_key_vec = frost_public_key_package.verifying_key().serialize().unwrap();
    let verifying_key = <&[u8; 32]>::try_from(verifying_key_vec.as_slice()).unwrap();

    let account_keys = derive_account_keys(verifying_key, &group_secret_key);

    let resp = get_requested_keys(&account_keys, key_type)?;
    drop(account_keys);

    send_apdu_chunks(comm, resp.as_slice())
}

#[inline(never)]
fn get_requested_keys(account_keys: &MultisigAccountKeys, key_type: &u8) -> Result<Vec<u8>, AppSW>{
    zlog_stack("start get_requested_keys\0");

    let mut resp: Vec<u8> = Vec::with_capacity(32 * 4);
    match key_type {
        0 => {
            let data = account_keys.public_address.public_address();
            resp.extend_from_slice(&data);

            Ok(resp)
        },
        1 => {
            resp.extend_from_slice(account_keys.view_key.authorizing_key.to_bytes().as_ref());
            resp.extend_from_slice(account_keys.view_key.nullifier_deriving_key.to_bytes().as_ref());
            resp.extend_from_slice(account_keys.incoming_viewing_key.view_key.as_ref());
            resp.extend_from_slice(account_keys.outgoing_viewing_key.view_key.as_ref());
            Ok(resp)
        },
        2 => {
            resp.extend_from_slice(account_keys.view_key.authorizing_key.to_bytes().as_ref());
            resp.extend_from_slice(account_keys.proof_authorizing_key.to_bytes().as_ref());
            Ok(resp)
        },
        _ => Err(AppSW::InvalidKeyType)
    }
}

#[inline(never)]
fn send_apdu_chunks(comm: &mut Comm, data: &[u8]) -> Result<(), AppSW> {
    zlog_stack("start send_apdu_chunks\0");

    let total_chunks = (data.len() + MAX_APDU_SIZE - 1) / MAX_APDU_SIZE;

    for (i, chunk) in data.chunks(MAX_APDU_SIZE).enumerate() {
        comm.append(chunk);

        if i < total_chunks - 1 {
            comm.reply_ok();
            match comm.next_event() {
                Event::Command(Instruction::DkgGetKeys {key_type: 0}) => {}
                _ => {},
            }
        }
    }

    Ok(())
}
