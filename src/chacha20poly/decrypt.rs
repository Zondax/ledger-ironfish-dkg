use crate::AppSW;
use aead::rand_core::RngCore;
use aead::AeadMut;
use alloc::vec::Vec;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use ledger_device_sdk::random::LedgerRng;

#[inline(never)]
pub fn decrypt(key: &[u8; 32], payload: &[u8]) -> Result<Vec<u8>, AppSW> {
    // Generate a random key
    let key = Key::clone_from_slice(key);

    // Create a ChaCha20Poly1305 instance
    let cipher = ChaCha20Poly1305::new(&key);

    let mut nonce_slice = payload[-12..];

    // Generate a random nonce
    let nonce = Nonce::clone_from_slice(&nonce_slice); // 96-bits; unique per message

    // Encrypt the message with associated data
    let mut ciphertext = cipher
        .decrypt(&nonce, payload)
        .map_err(|_| AppSW::EncryptionFail)?;

    Ok(ciphertext)
}
