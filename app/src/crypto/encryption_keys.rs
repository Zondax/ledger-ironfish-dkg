use blake2b_simd::Params as Blake2b;
use jubjub::AffinePoint;

use crate::{
    ironfish::{constants::SHARED_KEY_PERSONALIZATION, view_keys::OutgoingViewKey},
    parser::KEY_LENGTH,
};

/// Calculate the key used to encrypt the shared keys for an [`crate::outputs::OutputDescription`].
///
/// The shared keys are encrypted using the outgoing viewing key for the
/// spender (the person creating the note owned by the receiver). This gets
/// combined with hashes of the output values to make a key unique to, and
/// signed by, the output.
///
/// Naming is getting a bit far-fetched here because it's the keys used to
/// encrypt other keys. Keys, all the way down!
#[inline(never)]
pub fn calculate_key_for_encryption_keys(
    outgoing_view_key: &OutgoingViewKey,
    value_commitment: &AffinePoint,
    // note_commitment: &Scalar,
    note_commitment: &[u8; 32],
    // public_key: &SubgroupPoint,
    public_key: &[u8; KEY_LENGTH],
) -> [u8; 32] {
    let mut key_input = [0u8; 128];
    key_input[0..32].copy_from_slice(&outgoing_view_key.view_key);
    key_input[32..64].copy_from_slice(&value_commitment.to_bytes());
    // key_input[64..96].copy_from_slice(&note_commitment.to_bytes());
    key_input[64..96].copy_from_slice(note_commitment);
    key_input[96..128].copy_from_slice(public_key);

    Blake2b::new()
        .hash_length(32)
        .personal(SHARED_KEY_PERSONALIZATION)
        .hash(&key_input)
        .as_bytes()
        .try_into()
        .expect("has has incorrect length")
}
