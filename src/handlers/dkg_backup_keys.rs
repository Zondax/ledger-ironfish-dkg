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
use crate::nvm::dkg_keys::DkgKeys;
use crate::{AppSW, Instruction};
use alloc::vec::Vec;
use ledger_device_sdk::io::{Comm, Event};
use crate::bolos::{zlog, zlog_stack};
use crate::crypto::chacha20poly::{compute_key, encrypt};

const MAX_APDU_SIZE: usize = 253;

#[inline(never)]
pub fn handler_dkg_backup_keys(comm: &mut Comm) -> Result<(), AppSW> {
    zlog("start handler_dkg_backup_keys\0");

    let data = DkgKeys.load_all_raw()?;
    let key = compute_key();

    let resp = encrypt(&key, data)?;

    send_apdu_chunks(comm, resp)
}

#[inline(never)]
fn send_apdu_chunks(comm: &mut Comm, data_vec: Vec<u8>) -> Result<(), AppSW> {
    zlog_stack("start send_apdu_chunks\0");

    let data = data_vec.as_slice();
    let total_chunks = (data.len() + MAX_APDU_SIZE - 1) / MAX_APDU_SIZE;

    for (i, chunk) in data.chunks(MAX_APDU_SIZE).enumerate() {
        zlog_stack("iter send_apdu_chunks\0");
        comm.append(chunk);

        if i < total_chunks - 1 {
            zlog_stack("another send_apdu_chunks\0");
            comm.reply_ok();
            match comm.next_event() {
                Event::Command(Instruction::DkgBackupKeys) => {}
                _ => {}
            }
        }
    }

    Ok(())
}
