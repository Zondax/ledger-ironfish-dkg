use core::ptr::addr_of_mut;
use group::{
    cofactor::{CofactorCurve, CofactorCurveAffine, CofactorGroup},
    prime::PrimeGroup,
    Curve, Group,
};

use crate::{
    bolos::zlog_stack,
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

const MERKE_NOTE: &str = "605a3f9d31f10e6a68c5685b6121c7f69fb1aa4f64fd7d7ed30ec5cf2b1af681afe
a24c3ade7b7a740e05586575e2e15f827b5d1cafe62e15268896f37f848631f9acf55162ec1f8e11f666dc0d809b1
3bd8ed9ff06534c5edd3a57c73dd9bd58e1ed05c4b4311adbca1acc322166951f97a2128bb0b5c7ec0fe848b48fad
77d1f4b94cb4a699cfe74f9323c6e8cc3da0c48d88f767aec3b029d5ca1c738f260e909eb3d1f28c7481e7fc4357c
5b4ffe7140bf0196674ff4f843bb2b3ee1acdc46893cdd196f7cd280ed4c1c7a43dd9c2da5e7359112b8c50681045
70bc62305bebd8b728401c3e01549bd282cf60dc7e2fa690e7aa29e9f92340b6f3908ceb30291f1ff5103e913c9ef
a4341bc78d48dbe2ac880668ab4083ad199a19b86f5cdb46a1577a63b832247136eb7f2fa470646cc24c75fbbbb1e
467b7b399995ea04d123532a2ab3951";

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

// let value_commitment = read_point(&mut reader)?;
// let note_commitment = read_scalar(&mut reader)?;
// let ephemeral_public_key = read_point(&mut reader)?;
//
// let mut encrypted_note = [0; ENCRYPTED_NOTE_SIZE + aead::MAC_SIZE];
// reader.read_exact(&mut encrypted_note[..])?;
// let mut note_encryption_keys = [0; NOTE_ENCRYPTION_KEY_SIZE];
// reader.read_exact(&mut note_encryption_keys[..])?;
impl<'a> FromBytes<'a> for MerkleNote<'a> {
    #[inline(never)]
    fn from_bytes_into(
        input: &'a [u8],
        out: &mut core::mem::MaybeUninit<Self>,
    ) -> Result<&'a [u8], nom::Err<ParserError>> {
        zlog_stack("MerkleNote::from_bytes_into\n");
        let (rem, affine) = take(AFFINE_POINT_SIZE)(input)?;
        let affine = affine
            .try_into()
            .map_err(|_| ParserError::ValueOutOfRange)?;
        let value_commitment = parse_affine_point(affine)?;
        zlog_stack("MerkleNote::value_commitment\n");

        let (rem, raw_scalar) = take(32usize)(rem)?;
        let raw_scalar = array_ref!(raw_scalar, 0, 32);
        zlog_stack("raw_scalar\n");

        let note_commitment = Scalar::from_bytes(raw_scalar)
            .into_option()
            .ok_or(ParserError::UnexpectedValue)?;
        zlog_stack("MerkleNote::note_commitment\n");

        // parsing ephemeral pubkey which is a SubgroupPoint
        let (rem, raw_scalar) = take(32usize)(rem)?;
        let raw_scalar = array_ref!(raw_scalar, 0, 32);

        // ephemeral_public_key is a subgroupPoint
        // however, we are read it as an extended point due to lack of support
        // to compute it from bytes, ironfish uses a custom version of the jubjub
        // crate that is incompatible with our target.
        let ephemeral_public_key = parse_affine_point(raw_scalar)?;
        zlog_stack("MerkleNote::ephemeral_public_key\n");

        // encrypted_note
        let (rem, raw_note) = take(ENCRYPTED_NOTE_SIZE + MAC_SIZE)(rem)?;
        let encrypted_note = array_ref!(raw_note, 0, ENCRYPTED_NOTE_SIZE + MAC_SIZE);
        zlog_stack("MerkleNote::encrypted_note\n");

        // note_encryption_keys
        let (rem, encryption_keys) = take(NOTE_ENCRYPTION_KEY_SIZE)(rem)?;
        let note_encryption_keys = array_ref!(encryption_keys, 0, NOTE_ENCRYPTION_KEY_SIZE);
        zlog_stack("MerkleNote::note_encryption_keys\n");

        let out = out.as_mut_ptr();

        unsafe {
            addr_of_mut!((*out).value_commitment).write(value_commitment);
            addr_of_mut!((*out).note_commitment).write(note_commitment);
            addr_of_mut!((*out).ephemeral_public_key).write(ephemeral_public_key);
            addr_of_mut!((*out).encrypted_note).write(encrypted_note);
            addr_of_mut!((*out).note_encryption_keys).write(note_encryption_keys);
        }
        zlog_stack("MerkleNote::from_bytes_into ok\n");
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
