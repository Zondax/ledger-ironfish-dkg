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

use crate::bolos::zlog_stack;
use crate::context::TxContext;
use crate::nvm::dkg_keys::DkgKeys;
use crate::utils::response::save_result;
use crate::AppSW;
use alloc::vec::Vec;
use ledger_device_sdk::io::Comm;

#[inline(never)]
pub fn handler_dkg_get_identities(comm: &mut Comm, ctx: &mut TxContext) -> Result<(), AppSW> {
    zlog_stack("handler_dkg_get_identities\0");

    let identities = DkgKeys.load_identities()?;
    let resp: Vec<_> = identities.iter().flat_map(|s| s.serialize()).collect();

    let total_chunks = save_result(ctx, resp.as_slice())?;
    comm.append(&total_chunks);

    Ok(())
}
