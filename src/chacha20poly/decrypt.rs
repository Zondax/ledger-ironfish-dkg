use crate::chacha20poly::constants::NONCE_LEN;
use crate::utils::zlog_stack;
use crate::AppSW;
use alloc::vec::Vec;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};

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
