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
use crate::crypto::guards::IronfishSecretGuard;
use crate::ironfish::multisig::{derive_account_keys, MultisigAccountKeys};
#[cfg(feature = "ledger")]
use crate::nvm::dkg_keys::DkgKeys;
#[cfg(feature = "ledger")]
use crate::nvm::DkgKeysReader;
use crate::AppSW;
use alloc::vec;
use alloc::vec::Vec;
use core::ptr;
#[cfg(feature = "ledger")]
use ledger_device_sdk::ecc::{bip32_derive, ChainCode, CurvesId, Secret};

const ED25519_KEY_LEN: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstantKey {
    SpendingKeyGenerator,
    ProofGenerationKeyGenerator,
    PublicKeyGenerator,
}

#[cfg(feature = "ledger")]
pub(crate) fn derive_multisig_account(data: Option<&[u8]>) -> Result<MultisigAccountKeys, AppSW> {
    zlog_stack("start derive_multisig_account\0");

    let (group_secret_key, frost_public_key_package) = match data {
        Some(r) => {
            let gsk = DkgKeysReader::load_group_secret_key(r)?;
            let frost_package = DkgKeysReader::load_frost_public_key_package(r)?;
            (gsk, frost_package)
        }
        None => {
            let gsk = DkgKeys.load_group_secret_key()?;
            let frost_package = DkgKeys.load_frost_public_key_package()?;
            (gsk, frost_package)
        }
    };

    let verifying_key: [u8; 32] = frost_public_key_package
        .verifying_key()
        .serialize()
        .map_err(|_| AppSW::InvalidKeyType)?
        .as_slice()
        .try_into()
        .map_err(|_| AppSW::InvalidKeyType)?;

    Ok(derive_account_keys(&verifying_key, &group_secret_key))
}

#[inline(never)]
pub(crate) fn multisig_to_key_type(
    account_keys: &MultisigAccountKeys,
    key_type: u8,
) -> Result<Vec<u8>, AppSW> {
    zlog_stack("start get_requested_keys\0");

    let mut resp: Vec<u8> = Vec::with_capacity(32 * 4);
    match key_type {
        0 => {
            let data = account_keys.public_address.public_address();
            resp.extend_from_slice(&data);

            Ok(resp)
        }
        1 => {
            resp.extend_from_slice(account_keys.view_key.authorizing_key.to_bytes().as_ref());
            resp.extend_from_slice(
                account_keys
                    .view_key
                    .nullifier_deriving_key
                    .to_bytes()
                    .as_ref(),
            );
            resp.extend_from_slice(account_keys.incoming_viewing_key.view_key.as_ref());
            resp.extend_from_slice(account_keys.outgoing_viewing_key.view_key.as_ref());
            Ok(resp)
        }
        2 => {
            resp.extend_from_slice(account_keys.view_key.authorizing_key.to_bytes().as_ref());
            resp.extend_from_slice(account_keys.proof_authorizing_key.to_bytes().as_ref());
            Ok(resp)
        }
        _ => Err(AppSW::InvalidKeyType),
    }
}

#[cfg(feature = "ledger")]
#[inline(never)]
pub(crate) fn compute_dkg_secret(index: u8) -> IronfishSecretGuard {
    let index_1 = (index * 2) as u32;
    let index_2 = index_1 + 1;

    let path_0: Vec<u32> = vec![
        (0x80000000 | 0x2c),
        (0x80000000 | 0x53a),
        (0x80000000),
        (0x80000000),
        (0x80000000 | index_1),
    ];
    let path_1: Vec<u32> = vec![
        (0x80000000 | 0x2c),
        (0x80000000 | 0x53a),
        (0x80000000),
        (0x80000000),
        (0x80000000 | index_2),
    ];

    let mut secret_key_0 = Secret::<ED25519_KEY_LEN>::new();
    let mut secret_key_1 = Secret::<ED25519_KEY_LEN>::new();
    let mut cc: ChainCode = Default::default();

    // Ignoring 'Result' here because known to be valid
    let _ = bip32_derive(
        CurvesId::Ed25519,
        &path_0,
        secret_key_0.as_mut(),
        Some(cc.value.as_mut()),
    );
    let _ = bip32_derive(
        CurvesId::Ed25519,
        &path_1,
        secret_key_1.as_mut(),
        Some(cc.value.as_mut()),
    );

    let dkg_secret =
        IronfishSecretGuard::from_secret_keys(secret_key_0.as_ref(), secret_key_1.as_ref());

    // Zero out the memory of secret_key_0 and secret_key_1
    unsafe {
        ptr::write_bytes(&mut secret_key_0 as *mut Secret<ED25519_KEY_LEN>, 0, 1);
        ptr::write_bytes(&mut secret_key_1 as *mut Secret<ED25519_KEY_LEN>, 0, 1);
    }

    dkg_secret
}
