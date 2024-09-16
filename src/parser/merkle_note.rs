use core::ptr::addr_of_mut;
use group::{
    cofactor::{CofactorCurve, CofactorCurveAffine, CofactorGroup},
    prime::PrimeGroup,
    Curve, Group,
};

use crate::{
    crypto::{calculate_key_for_encryption_keys, parse_affine_point, read_fr, read_scalar},
    ironfish::{
        errors::IronfishError,
        view_keys::{shared_secret, OutgoingViewKey},
    },
    FromBytes,
};
use crate::{
    crypto::{decrypt, parse_extended_point},
    ironfish::public_address::PublicAddress,
    parser::{
        ParserError, AFFINE_POINT_SIZE, ENCRYPTED_NOTE_SIZE, MAC_SIZE, NOTE_ENCRYPTION_KEY_SIZE,
    },
};
use arrayref::array_ref;
use jubjub::AffinePoint;
use jubjub::{ExtendedPoint, Scalar, SubgroupPoint};
use nom::bytes::complete::take;

use super::{Note, ENCRYPTED_SHARED_KEY_SIZE};

#[derive(Clone, Debug)]
pub struct MerkleNote<'a> {
    /// Randomized value commitment. Sometimes referred to as
    /// `cv` in the literature. It's calculated by multiplying a value by a
    /// random number. Commits this note to the value it contains
    /// without revealing what that value is.
    /// Use AffinePoint instead of ExtendedPoint
    /// for simplicity and easy conversion from/to bytes
    pub(crate) value_commitment: AffinePoint,

    /// The hash of the note, committing to it's internal state
    pub(crate) note_commitment: Scalar,

    /// Public part of ephemeral diffie-hellman key-pair. See the discussion on
    /// [`shared_secret`] to understand how this is used
    // pub(crate) ephemeral_public_key: SubgroupPoint,
    // We use AffinePoint because it is more compact and memory efficient
    // we do not have the right API to parse bytes into a SubgroupPoint
    pub(crate) ephemeral_public_key: AffinePoint,

    /// note as encrypted by the diffie hellman public key
    /// we use this data to decrypt a Note as represented by:
    /// https://github.com/iron-fish/ironfish/blob/master/ironfish-rust/src/note.rs#L88
    pub(crate) encrypted_note: &'a [u8; ENCRYPTED_NOTE_SIZE + MAC_SIZE],

    /// Keys used to encrypt the note. These are stored in encrypted format
    /// using the spender's outgoing viewing key, and allow the spender to
    /// decrypt it. The receiver (owner) doesn't need these, as they can decrypt
    /// the note directly using their incoming viewing key.
    pub(crate) note_encryption_keys: &'a [u8; NOTE_ENCRYPTION_KEY_SIZE],
}
impl<'a> FromBytes<'a> for MerkleNote<'a> {
    #[inline(never)]
    fn from_bytes_into(
        input: &'a [u8],
        out: &mut core::mem::MaybeUninit<Self>,
    ) -> Result<&'a [u8], nom::Err<ParserError>> {
        let (rem, affine) = take(AFFINE_POINT_SIZE)(input)?;
        let affine = affine
            .try_into()
            .map_err(|_| ParserError::ValueOutOfRange)?;
        let value_commitment = parse_affine_point(affine)?;

        let (rem, raw_scalar) = take(3232usize)(rem)?;
        let raw_scalar = array_ref!(raw_scalar, 0, 32);

        let note_commitment = Scalar::from_bytes(raw_scalar)
            .into_option()
            .ok_or(ParserError::UnexpectedValue)?;

        // parsing ephemeral pubkey which is a SubgroupPoint
        let (rem, raw_scalar) = take(32usize)(rem)?;
        let raw_scalar = array_ref!(raw_scalar, 0, 32);

        // ephemeral_public_key is a subgroupPoint
        // however, we are read it as an extended point due to lack of support
        // to compute it from bytes, ironfish uses a custom version of the jubjub
        // crate that is incompatible with our target.
        let ephemeral_public_key = parse_affine_point(raw_scalar)?;

        // encrypted_note
        let (rem, raw_note) = take(ENCRYPTED_NOTE_SIZE + MAC_SIZE)(rem)?;
        let encrypted_note = array_ref!(raw_note, 0, ENCRYPTED_NOTE_SIZE + MAC_SIZE);

        // note_encryption_keys
        let (rem, encryption_keys) = take(NOTE_ENCRYPTION_KEY_SIZE)(rem)?;
        let note_encryption_keys = array_ref!(encryption_keys, 0, NOTE_ENCRYPTION_KEY_SIZE);

        let out = out.as_mut_ptr();

        unsafe {
            addr_of_mut!((*out).value_commitment).write(value_commitment);
            addr_of_mut!((*out).note_commitment).write(note_commitment);
            addr_of_mut!((*out).ephemeral_public_key).write(ephemeral_public_key);
            addr_of_mut!((*out).encrypted_note).write(encrypted_note);
            addr_of_mut!((*out).note_encryption_keys).write(note_encryption_keys);
        }
        Ok(rem)
    }
}

impl<'a> MerkleNote<'a> {
    pub fn decrypt_note_for_spender(
        &self,
        spender_key: &OutgoingViewKey,
    ) -> Result<Note, IronfishError> {
        let encryption_key = calculate_key_for_encryption_keys(
            spender_key,
            &self.value_commitment,
            &self.note_commitment,
            &self.ephemeral_public_key.to_bytes(),
        );

        let note_encryption_keys: [u8; ENCRYPTED_SHARED_KEY_SIZE] =
            decrypt(&encryption_key, self.note_encryption_keys)?;

        let public_address = PublicAddress::new(&note_encryption_keys[..32].try_into().unwrap())?;

        let (rem, secret_key) =
            read_fr(&note_encryption_keys[32..]).map_err(|_| IronfishError::InvalidScalar)?;
        let shared_key = shared_secret(&secret_key, &public_address.0, &self.ephemeral_public_key);
        let note =
            Note::from_spender_encrypted(public_address.0, &shared_key, &self.encrypted_note)?;

        // FIXME: Verify the node commitment
        // note.verify_commitment(self.note_commitment)?;

        Ok(note)
    }
}
