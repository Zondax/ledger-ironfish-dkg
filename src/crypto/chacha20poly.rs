use crate::bolos::zlog_stack;
use crate::AppSW;
use aead::rand_core::RngCore;
use alloc::vec;
use alloc::vec::Vec;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use ledger_device_sdk::ecc::{bip32_derive, ChainCode, CurvesId, Secret};
use ledger_device_sdk::random::LedgerRng;

pub const NONCE_LEN: usize = 12;
pub const KEY_LEN: usize = 32;

#[inline(never)]
pub fn decrypt(key: &[u8; 32], payload: &[u8], nonce: &[u8]) -> Result<Vec<u8>, AppSW> {
    zlog_stack("start decrypt\0");

    // Generate a random key
    let key = Key::clone_from_slice(key);

    // Create a ChaCha20Poly1305 instance
    let cipher = ChaCha20Poly1305::new(&key);

    let nonce_slice = <&[u8; NONCE_LEN]>::try_from(nonce).map_err(|_| AppSW::InvalidPayload)?;

    // Generate a random nonce
    let nonce = Nonce::clone_from_slice(nonce_slice); // 96-bits; unique per message

    // Encrypt the message with associated data
    let mut ciphertext = cipher
        .decrypt(&nonce, payload)
        .map_err(|_| AppSW::DecryptionFail)?;

    Ok(ciphertext)
}

#[inline(never)]
pub fn encrypt(key: &[u8; KEY_LEN], payload: &[u8]) -> Result<Vec<u8>, AppSW> {
    let mut rng = LedgerRng {};
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

#[inline(never)]
pub fn compute_key() -> [u8; KEY_LEN] {
    let path_0: Vec<u32> = vec![
        (0x80000000 | 0x2c),
        (0x80000000 | 0x53a),
        (0x80000000 | 0x0),
        (0x80000000 | 0x0),
        (0x80000000 | 0x0),
    ];

    let mut secret_key_0 = Secret::<64>::new();
    let mut cc: ChainCode = Default::default();

    // Ignoring 'Result' here because known to be valid
    let _ = bip32_derive(
        CurvesId::Ed25519,
        &path_0,
        secret_key_0.as_mut(),
        Some(cc.value.as_mut()),
    );

    secret_key_0.as_ref()[0..KEY_LEN].try_into().unwrap()
}
