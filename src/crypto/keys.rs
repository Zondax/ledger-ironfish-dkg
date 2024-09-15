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
use crate::ironfish::multisig::{derive_account_keys, MultisigAccountKeys};
use crate::{nvm::dkg_keys::DkgKeys, AppSW};
use alloc::vec::Vec;
use ironfish_frost::dkg::group_key::{GroupSecretKey, GROUP_SECRET_KEY_LEN};
use ironfish_frost::dkg::round3::PublicKeyPackage;
use ironfish_frost::frost::keys::PublicKeyPackage as FrostPublicKeyPackage;
use ledger_device_sdk::io::{Comm, Event};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstantKey {
    SpendingKeyGenerator,
    ProofGenerationKeyGenerator,
    PublicKeyGenerator,
}

pub(crate) fn get_dkg_keys() -> Result<MultisigAccountKeys, AppSW> {
    zlog_stack("start handler_dkg_get_keys\0");

    let group_secret_key = DkgKeys.load_group_secret_key()?;
    let frost_public_key_package = DkgKeys.load_frost_public_key_package()?;

    let verifying_key: [u8; 32] = frost_public_key_package
        .verifying_key()
        .serialize()
        .map_err(|_| AppSW::InvalidKeyType)?
        .as_slice()
        .try_into()
        .map_err(|_| AppSW::InvalidKeyType)?;

    Ok(derive_account_keys(&verifying_key, &group_secret_key))
}
