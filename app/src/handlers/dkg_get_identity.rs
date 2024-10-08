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

use crate::app_ui::run_action::ui_review_get_identity;
use crate::bolos::zlog_stack;
use crate::crypto::compute_dkg_secret;
use crate::AppSW;
use ledger_device_sdk::io::Comm;

const MAX_IDENTITY_INDEX: u8 = 5;

#[inline(never)]
pub fn handler_dkg_get_identity(comm: &mut Comm, require_review: bool) -> Result<(), AppSW> {
    zlog_stack("start handler_identity\0");

    let data_vec = comm
        .get_data()
        .map_err(|_| AppSW::WrongApduLength)?
        .to_vec();
    let data = data_vec.as_slice();

    if data.len() != 1 || data[0] > MAX_IDENTITY_INDEX {
        return Err(AppSW::InvalidIdentityIndex);
    }

    let secret = compute_dkg_secret(data[0]);
    let identity = secret.to_identity();

    if require_review && !ui_review_get_identity(data[0])? {
        return Err(AppSW::Deny);
    }

    comm.append(identity.serialize().as_ref());

    Ok(())
}
