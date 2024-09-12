use crate::AppSW;
use aead::rand_core::RngCore;
use alloc::vec::Vec;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use ledger_device_sdk::random::LedgerRng;

pub fn encrypt(key: &[u8; 32], payload: &[u8]) -> Result<Vec<u8>, AppSW> {
    let mut rng = LedgerRng {};
    let v1 = rng.next_u64();
    let v2 = rng.next_u64();

    // Generate a random key
    let key = Key::clone_from_slice(key);

    // Create a ChaCha20Poly1305 instance
    let cipher = ChaCha20Poly1305::new(&key);

    let mut nonce_slice = [0u8; 12];
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
