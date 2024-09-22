use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce};
use core::convert::TryFrom;
use ff::PrimeField;
use group::cofactor::CofactorGroup;
use group::Group;
use jubjub::{AffinePoint, ExtendedPoint, Fq, Fr, Scalar};
use nom::bytes::complete::take;

use crate::ironfish::constants::{
    PROOF_GENERATION_KEY_GENERATOR, PUBLIC_KEY_GENERATOR, SPENDING_KEY_GENERATOR,
};
use crate::ironfish::errors::IronfishError;
use crate::{parser::ParserError, ConstantKey};

pub fn parse_affine_point(raw_bytes: &[u8; 32]) -> Result<AffinePoint, ParserError> {
    AffinePoint::from_bytes(*raw_bytes)
        .into_option()
        .ok_or(ParserError::UnexpectedValue)
}

pub fn parse_extended_point(raw_bytes: &[u8; 32]) -> Result<ExtendedPoint, ParserError> {
    parse_affine_point(raw_bytes).map(ExtendedPoint::from)
}

/// Decrypt the encrypted text using the given key and ciphertext, also checking
/// that the mac tag is correct.

pub(crate) fn decrypt<const SIZE: usize>(
    key: &[u8; 32],
    ciphertext: &[u8],
) -> Result<[u8; SIZE], IronfishError> {
    use chacha20poly1305::AeadInPlace;

    let decryptor = ChaCha20Poly1305::new(Key::from_slice(key));

    let mut plaintext = [0u8; SIZE];
    plaintext.copy_from_slice(&ciphertext[..SIZE]);

    decryptor
        .decrypt_in_place_detached(
            &Nonce::default(),
            &[],
            &mut plaintext,
            ciphertext[SIZE..].into(),
        )
        .map_err(|_| IronfishError::InvalidDecryptionKey)?;

    Ok(plaintext)
}

/// Reads a PrimeField element from a byte array, valid PrimeField elements are:
/// Fr: Fr::Repr is [u8; 32]
/// Scalar: Scalar::Repr is [u8; 32]
/// Fp: Fp::Repr is [u8; 32]
/// Fq: Fq::Repr is [u8; 32]
macro_rules! generate_from_bytes_conversion {
    ($type:ty, $func_name:ident) => {
        pub fn $func_name(bytes: &[u8]) -> Result<(&[u8], $type), ParserError> {
            let (rem, raw) = take(32usize)(bytes)?;
            let bytes = arrayref::array_ref!(raw, 0, 32);
            <$type>::from_bytes(bytes)
                .into_option()
                .ok_or(ParserError::InvalidScalar)
                .map(|f| (rem, f))
        }
    };
}

// Generates functions
generate_from_bytes_conversion!(Fr, read_fr);
generate_from_bytes_conversion!(Fq, read_fq);
generate_from_bytes_conversion!(Scalar, read_scalar);

#[cfg(test)]
mod utils {
    use super::*;

    const EXTENDED_POINT: &str = "247f750514f0a0018af8fc17ef85ad376fa92390603bf9f8b8cb1597d57d7d52";

    #[test]
    fn parse_extended_as_affine() {
        let raw_extended = hex::decode(EXTENDED_POINT).unwrap();
        let raw_extended: [u8; 32] = raw_extended.try_into().unwrap();

        let affine = parse_affine_point(&raw_extended).unwrap();

        assert_eq!(raw_extended, affine.to_bytes());
    }
}
