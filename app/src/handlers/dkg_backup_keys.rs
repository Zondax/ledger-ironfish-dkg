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
use crate::app_ui::run_action::ui_review_backup_keys;
use crate::bolos::zlog;
use crate::context::TxContext;
use crate::crypto::chacha20poly::{compute_key, encrypt};
use crate::crypto::{derive_multisig_account, multisig_to_key_type};
use crate::nvm::dkg_keys::DkgKeys;
use crate::utils::response::save_result;
use crate::AppSW;
use ledger_device_sdk::io::Comm;

#[inline(never)]
pub fn handler_dkg_backup_keys(comm: &mut Comm, ctx: &mut TxContext) -> Result<(), AppSW> {
    zlog("start handler_dkg_backup_keys\0");

    let account_keys = derive_multisig_account(None)?;
    let public_address = multisig_to_key_type(&account_keys, 0u8)?;
    drop(account_keys);

    let min_signers = DkgKeys.load_min_signers()?;
    let participants = DkgKeys.load_identities()?.len();

    if !ui_review_backup_keys(public_address, participants as u8, min_signers as u8)? {
        return Err(AppSW::Deny);
    }

    let data = DkgKeys.backup_keys()?;
    let key = compute_key();

    let resp = encrypt(&key, data.as_slice())?;

    let total_chunks = save_result(ctx, resp.as_slice())?;
    comm.append(&total_chunks);

    Ok(())
}
