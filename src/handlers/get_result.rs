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

use crate::AppSW;
use ledger_device_sdk::io::Comm;
use crate::context::TxContext;

const MAX_APDU_SIZE: usize = 253;

#[inline(never)]
pub fn handler_get_result(comm: &mut Comm, ctx: &mut TxContext, page: u8) -> Result<(), AppSW> {
    let start_page_pos:usize = page as usize * MAX_APDU_SIZE;
    let mut end_page_pos:usize = start_page_pos + MAX_APDU_SIZE;

    if ctx.buffer.pos < end_page_pos{
        end_page_pos = ctx.buffer.pos;
    }

    let data_to_send = ctx.buffer.get_slice(start_page_pos, end_page_pos)?;
    comm.append(data_to_send);

    Ok(())
}