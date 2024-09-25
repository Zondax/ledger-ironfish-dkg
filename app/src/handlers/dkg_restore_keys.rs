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
use crate::accumulator::accumulate_data;
use crate::app_ui::run_action::ui_review_restore_keys;
use crate::bolos::zlog_stack;
use crate::context::TxContext;
use crate::crypto::chacha20poly::{compute_key, decrypt, NONCE_LEN};
use crate::nvm::dkg_keys::DkgKeys;
use crate::nvm::DkgKeysReader;
use crate::AppSW;
use alloc::vec;
use alloc::vec::Vec;
use ledger_device_sdk::io::Comm;

#[inline(never)]
pub fn handler_dkg_restore_keys(
    comm: &mut Comm,
    chunk: u8,
    ctx: &mut TxContext,
) -> Result<(), AppSW> {
    zlog_stack("start handler_dkg_restore_keys\0");

    accumulate_data(comm, chunk, ctx)?;
    if !ctx.done {
        return Ok(());
    }

    if ctx.buffer.pos < NONCE_LEN {
        return Err(AppSW::InvalidPayload);
    }

    let split_pos = ctx.buffer.pos - NONCE_LEN;
    let data = ctx.buffer.get_slice(0, split_pos)?;
    let nonce = ctx.buffer.get_slice(split_pos, ctx.buffer.pos)?;

    let key = compute_key();

    let resp: Vec<u8> = decrypt(&key, data, nonce)?;

    review_restore_keys(resp.as_slice().as_ref())?;

    DkgKeys.restore_keys(resp.as_slice())
}

#[inline(never)]
fn review_restore_keys(data: &[u8]) -> Result<(), AppSW> {
    let dkg_keys_reader = DkgKeysReader::new(data);

    //account_keys = get_dkg_keys()?;
    let public_address = vec![0, 1, 2, 3, 4, 5];
    //drop(account_keys);

    let min_signers = dkg_keys_reader.load_min_signers()?;
    let participants = dkg_keys_reader.load_identities()?.len();

    if !ui_review_restore_keys(public_address, participants as u8, min_signers as u8)? {
        return Err(AppSW::Deny);
    }

    Ok(())
}
