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
use crate::crypto::{generate_key_type, get_dkg_keys};
use crate::ironfish::multisig::{derive_account_keys, MultisigAccountKeys};
use crate::nvm::dkg_keys::DkgKeys;
use crate::AppSW;
use alloc::vec::Vec;
use ledger_device_sdk::io::Comm;

#[inline(never)]
pub fn handler_dkg_get_keys(
    comm: &mut Comm,
    key_type: &u8,
    _ctx: &mut TxContext,
) -> Result<(), AppSW> {
    zlog_stack("start handler_dkg_get_keys\0");

    let account_keys = get_dkg_keys()?;
    let resp = generate_key_type(&account_keys, *key_type)?;
    drop(account_keys);

    comm.append(resp.as_slice().as_ref());
    Ok(())
}
