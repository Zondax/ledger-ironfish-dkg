use blake2b_simd::Params as Blake2b;

const PERSONALIZATION: &[u8; 16] = b"Iron Fish Money ";

use jubjub::{ExtendedPoint, Fr as Scalar};

use crate::parser::{ParserError, KEY_LENGTH};

// As defined by:
// https://ironfish.network/learn/whitepaper/protocol/accounts
#[derive(Clone, Debug)]
pub struct OutgoingViewKey([u8; KEY_LENGTH]);

impl OutgoingViewKey {
    pub fn from_secret(secret_key: &Scalar) -> Result<Self, ParserError> {
        // 64-bytes as per documentation
        let result: [u8; 2 * KEY_LENGTH] = Blake2b::new()
            .hash_length(64)
            .personal(PERSONALIZATION)
            .to_state()
            .update(&secret_key.to_bytes())
            .update(&[2])
            .finalize()
            .as_bytes()
            .try_into()
            .map_err(|_| ParserError::InvalidKey)?;

        // Take the first 32 bytes (256 bits) of the 64-byte (512-bit) hash
        let mut ovk = [0u8; 32];
        ovk.copy_from_slice(&result[..32]);

        Ok(Self(ovk))
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }
}
