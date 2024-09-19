use core::mem::MaybeUninit;

use crate::app_ui::run_action::ui_review_transaction;
use crate::crypto::get_dkg_keys;
use crate::utils::response::save_result;
use crate::{AppSW, FromBytes, Transaction};
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
use crate::bolos::zlog_stack;
use crate::context::TxContext;
use crate::ironfish::constants::TX_HASH_LEN;
use crate::nvm::buffer::{Buffer, BUFFER_SIZE};
use ledger_device_sdk::io::Comm;
use nom::bytes::complete::take;
use nom::number::complete::{be_u16, be_u32};

#[cfg(not(any(target_os = "stax", target_os = "flex")))]
#[inline(never)]
pub fn handler_review_tx(comm: &mut Comm, chunk: u8, ctx: &mut TxContext) -> Result<(), AppSW> {
    use crate::nvm::set_tx_hash;

    zlog_stack("start handler_review_tx\0");

    accumulate_data(comm, chunk, ctx)?;
    if !ctx.done {
        return Ok(());
    }
    zlog_stack("Accumulator done\0");

    // lets get access to all buffer raw data
    // because we would handle offests internally in our
    // transaction parser
    let input = ctx.buffer.get_full_buffer();
    zlog_stack("got_buffer\0");

    let mut tx = MaybeUninit::uninit();

    zlog_stack("start tx_parsing\0");
    Transaction::from_bytes_into(input, &mut tx).map_err(|_| AppSW::TxParsingFail)?;

    let tx = unsafe { tx.assume_init() };
    zlog_stack("done tx_parsing\0");
    let hash = tx.hash();

    // Get outgoing viewing key
    let account_keys = get_dkg_keys()?;

    // review transaction
    if !ui_review_transaction(&tx, &account_keys.outgoing_viewing_key)? {
        return Err(AppSW::Deny);
    }

    // Save transaction hash in memory
    set_tx_hash(hash);

    let total_chunks = save_result(ctx, hash.as_slice())?;
    comm.append(&total_chunks);

    Ok(())
}

// For now lets keep Transaction parsing and review out of the
// command processing flow
#[cfg(any(target_os = "stax", target_os = "flex"))]
#[inline(never)]
pub fn handler_review_tx(comm: &mut Comm, chunk: u8, ctx: &mut TxContext) -> Result<(), AppSW> {
    zlog_stack("start handler_review_tx\0");

    let mut dummy = [0u8; TX_HASH_LEN];

    let total_chunks = save_result(ctx, dummy.as_slice())?;
    comm.append(&total_chunks);

    Ok(())
}
