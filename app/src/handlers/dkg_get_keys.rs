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
use crate::app_ui::ui_review_get_keys;
use crate::bolos::zlog_stack;
use crate::crypto::{compute_dkg_secret, derive_multisig_account, multisig_to_key_type};
use crate::nvm::dkg_keys::DkgKeys;
use crate::AppSW;
use alloc::vec::Vec;
use core::ptr;
use ledger_device_sdk::io::Comm;

#[inline(never)]
pub fn handler_dkg_get_keys(comm: &mut Comm, review: bool, key_type: u8) -> Result<(), AppSW> {
    zlog_stack("start handler_dkg_get_keys\0");

    let mut resp: Vec<u8>;

    if key_type == 3 {
        let identity_index = DkgKeys.load_identity_index()?;
        let identity = compute_dkg_secret(identity_index as u8).to_identity();
        resp = identity.serialize().as_slice().to_vec();

        if review && !ui_review_get_identity(identity_index as u8)? {
            return Err(AppSW::Deny);
        }
    } else {
        let account_keys = derive_multisig_account(None)?;
        resp = multisig_to_key_type(&account_keys, key_type)?;
        drop(account_keys);

        if review && !ui_review_get_keys(&resp, key_type)? {
            return Err(AppSW::Deny);
        }
    }

    comm.append(resp.as_slice().as_ref());

    // Zero out memory for the response data
    unsafe {
        ptr::write_bytes(&mut resp as *mut Vec<u8>, 0, 1);
    }

    Ok(())
}
