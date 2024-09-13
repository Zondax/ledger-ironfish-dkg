use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce};
use core::convert::TryFrom;
use ff::PrimeField;
use group::Group;
use group::{cofactor::CofactorGroup, GroupEncoding};
use jubjub::{AffinePoint, ExtendedPoint, Fq, Fr, Scalar};
use nom::bytes::complete::take;

use crate::ironfish::constants::{
    PROOF_GENERATION_KEY_GENERATOR, PUBLIC_KEY_GENERATOR, SPENDING_KEY_GENERATOR,
};
use crate::ironfish::errors::IronfishError;
use crate::{parser::ParserError, ConstantKey};

#[inline(never)]
pub fn from_bytes_wide(input: &[u8; 64], output: &mut [u8; 32]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let result = Fr::from_bytes_wide(input).to_bytes();
    output.copy_from_slice(&result[0..32]);

    output
}

#[inline(never)]
pub fn scalar_multiplication(input: &[u8; 32], key: ConstantKey) -> [u8; 32] {
    let key_point = match key {
        ConstantKey::SpendingKeyGenerator => SPENDING_KEY_GENERATOR,
        ConstantKey::ProofGenerationKeyGenerator => PROOF_GENERATION_KEY_GENERATOR,
        ConstantKey::PublicKeyGenerator => PUBLIC_KEY_GENERATOR,
    };

    let extended_point = key_point.multiply_bits(input);
    let result = AffinePoint::from(&extended_point);

    let mut output = [0u8; 32];
    output.copy_from_slice(&result.to_bytes());

    output
}

#[inline(never)]
pub fn randomize_key(key: &[u8; 32], randomness: &[u8; 32]) -> Result<[u8; 32], ParserError> {
    let mut output = [0u8; 32];

    let mut skfr = Fr::from_bytes(key)
        .into_option()
        .ok_or(ParserError::UnexpectedValue)?;

    // Safe to unwrap
    let alphafr = Fr::from_bytes(randomness)
        .into_option()
        .ok_or(ParserError::UnexpectedValue)?;

    skfr += alphafr;

    output.copy_from_slice(&skfr.to_bytes());

    Ok(output)
}

#[inline(never)]
pub fn compute_sbar(s: &[u8; 32], r: &[u8; 32], rsk: &[u8; 32]) -> Result<[u8; 32], ParserError> {
    let s_point = Fr::from_bytes(s)
        .into_option()
        .ok_or(ParserError::UnexpectedValue)?;

    let r_point = Fr::from_bytes(r)
        .into_option()
        .ok_or(ParserError::UnexpectedValue)?;

    let rsk_point = Fr::from_bytes(rsk)
        .into_option()
        .ok_or(ParserError::UnexpectedValue)?;

    let mut sbar = [0u8; 32];
    let sbar_tmp = r_point + s_point * rsk_point;

    sbar.copy_from_slice(&sbar_tmp.to_bytes());

    Ok(sbar)
}

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
