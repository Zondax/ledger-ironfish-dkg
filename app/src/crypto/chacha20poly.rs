use crate::crypto::guards::EncryptionKeyGuard;
use crate::{bolos::zlog_stack, rand::LedgerRng, AppSW};
use alloc::vec;
use alloc::vec::Vec;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use core::ptr;
#[cfg(feature = "ledger")]
use ledger_device_sdk::ecc::{bip32_derive, ChainCode, CurvesId, Secret};

use super::guards::KeysDataGuard;
// #[cfg(feature = "ledger")]
// use ledger_device_sdk::random::LedgerRng;

pub const NONCE_LEN: usize = 12;
const SECRET_KEY_LEN: usize = 32;
const ED25519_KEY_LEN: usize = 64;

#[inline(never)]
pub fn decrypt(key: &[u8; 32], payload: &[u8], nonce: &[u8]) -> Result<KeysDataGuard, AppSW> {
    zlog_stack("start decrypt\0");

    // Generate a random key
    let key = Key::clone_from_slice(key);

    // Create a ChaCha20Poly1305 instance
    let cipher = ChaCha20Poly1305::new(&key);

    let nonce_slice = <&[u8; NONCE_LEN]>::try_from(nonce).map_err(|_| AppSW::InvalidPayload)?;

    // Generate a random nonce
    let nonce = Nonce::clone_from_slice(nonce_slice); // 96-bits; unique per message

    // Encrypt the message with associated data
    let ciphertext = cipher
        .decrypt(&nonce, payload)
        .map_err(|_| AppSW::DecryptionFail)?;

    Ok(KeysDataGuard::new(ciphertext))
}

#[inline(never)]
pub fn encrypt(key: &[u8; SECRET_KEY_LEN], payload: &[u8]) -> Result<Vec<u8>, AppSW> {
    let mut rng = LedgerRng::new();
    let v1 = rng.next_u64();
    let v2 = rng.next_u64();

    // Generate a random key
    let key = Key::clone_from_slice(key);

    // Create a ChaCha20Poly1305 instance
    let cipher = ChaCha20Poly1305::new(&key);

    let mut nonce_slice = [0u8; NONCE_LEN];
    nonce_slice[..8].copy_from_slice(&v1.to_be_bytes());
    nonce_slice[8..12].copy_from_slice(&v2.to_be_bytes()[0..4]);

    // Generate a random nonce
    let nonce = Nonce::clone_from_slice(&nonce_slice); // 96-bits; unique per message

    // Encrypt the message with associated data
    let mut ciphertext = cipher
        .encrypt(&nonce, payload)
        .map_err(|_| AppSW::EncryptionFail)?;
    let mut nonce_vec = nonce_slice.to_vec();
    ciphertext.append(&mut nonce_vec);

    Ok(ciphertext)
}

#[cfg(feature = "ledger")]
#[inline(never)]
pub fn compute_key() -> EncryptionKeyGuard {
    let path_0: Vec<u32> = vec![
        (0x80000000 | 0x2c),
        (0x80000000 | 0x53a),
        (0x80000000),
        (0x80000000),
        (0x80000000),
    ];

    let mut secret_key_0 = Secret::<ED25519_KEY_LEN>::new();
    let mut cc: ChainCode = Default::default();

    // Ignoring 'Result' here because known to be valid
    let _ = bip32_derive(
        CurvesId::Ed25519,
        &path_0,
        secret_key_0.as_mut(),
        Some(cc.value.as_mut()),
    );

    let key = EncryptionKeyGuard::from_secret_keys(secret_key_0.as_ref());

    // Zero out the memory of secret_key_0 and secret_key_1
    unsafe {
        ptr::write_bytes(&mut secret_key_0 as *mut Secret<ED25519_KEY_LEN>, 0, 1);
    }

    key
}
