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
use crate::AppSW;
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

    let resp = decrypt(&key, data, nonce)?;

    if !ui_review_restore_keys()? {
        return Err(AppSW::Deny);
    }

    DkgKeys.restore_keys(resp.as_slice())
}
