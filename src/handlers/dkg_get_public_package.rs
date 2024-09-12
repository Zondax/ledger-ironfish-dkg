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
pub fn handler_dkg_get_public_package(
    comm: &mut Comm
) -> Result<(), AppSW> {
    zlog_stack("start handler_dkg_get_public_package\0");

    let identities = DkgKeys.load_identities()?;
    let min_signers = DkgKeys.load_min_signers()?;
    let frost_public_key_package = DkgKeys.load_frost_public_key_package()?;

    let p = PublicKeyPackage::from_frost(frost_public_key_package, identities, min_signers as u16);

    let resp = p.serialize();

    send_apdu_chunks(comm, resp.as_slice())
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
