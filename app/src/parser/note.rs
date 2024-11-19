use alloc::{string::String, vec::Vec};
use core::{mem::MaybeUninit, ptr::addr_of_mut};
use jubjub::AffinePoint;
use nom::number::complete::le_u64;

#[cfg(feature = "ledger")]
use crate::nvm::settings::Settings;
use crate::{
    bolos::zlog_stack,
    crypto::{decrypt, read_scalar},
    ironfish::{errors::IronfishError, public_address::PublicAddress},
    parser::AssetIdentifier,
    token::TokenList,
    utils::int_format::{token_to_fp_str, u64_to_str},
    FromBytes,
};

use super::{memo::Memo, ParserError, ENCRYPTED_NOTE_SIZE, MAC_SIZE};

/// A note (think bank note) represents a value in the owner's "account".
/// When spending, proof that the note exists in the tree needs to be provided,
/// along with a nullifier key that is made public so the owner cannot attempt
/// to spend that note again.447903
///
/// When receiving funds, a new note needs to be created for the new owner
/// to hold those funds.
pub struct Note {
    /// Asset identifier the note is associated with
    pub(crate) asset_id: AssetIdentifier,

    /// A public address for the owner of the note.
    pub(crate) owner: PublicAddress,

    /// Value this note represents.
    pub(crate) value: u64,

    /// A random value generated when the note is constructed.
    /// This helps create zero knowledge around the note,
    /// allowing the owner to prove they have the note without revealing
    /// anything else about it.
    pub(crate) randomness: jubjub::Fr,

    /// Arbitrary note the spender can supply when constructing a spend so the
    /// receiver has some record from whence it came.
    /// Note: While this is encrypted with the output, it is not encoded into
    /// the proof in any way.
    pub(crate) memo: Memo,

    /// A public address for the sender of the note.
    pub(crate) sender: PublicAddress,
}

impl Note {
    /// Create a note from its encrypted representation, given the spender's
    /// view key.
    ///
    /// The note is stored on the [`crate::outputs::OutputDescription`] in
    /// encrypted form. The spender encrypts it when they construct the output
    /// using a shared secret derived from the owner's public key.
    ///
    /// This function allows the owner to decrypt the note using the derived
    /// shared secret and their own view key.
    #[inline(never)]
    pub(crate) fn from_spender_encrypted(
        // public_address: SubgroupPoint,
        public_address: AffinePoint,
        shared_secret: &[u8; 32],
        encrypted_bytes: &[u8; ENCRYPTED_NOTE_SIZE + MAC_SIZE],
    ) -> Result<Self, IronfishError> {
        let mut this = MaybeUninit::uninit();
        Note::decrypt_note_parts(shared_secret, encrypted_bytes, &mut this)
            .map_err(|_| IronfishError::FailedXChaCha20Poly1305Decryption)?;

        let owner = PublicAddress(public_address);

        let out = this.as_mut_ptr();
        unsafe {
            addr_of_mut!((*out).owner).write(owner);
        }

        Ok(unsafe { this.assume_init() })
    }

    #[inline(never)]
    fn decrypt_note_parts(
        shared_secret: &[u8; 32],
        encrypted_bytes: &[u8; ENCRYPTED_NOTE_SIZE + MAC_SIZE],
        out: &mut MaybeUninit<Self>,
    ) -> Result<(), ParserError> {
        let out = out.as_mut_ptr();

        let plaintext_bytes: [u8; ENCRYPTED_NOTE_SIZE] =
            decrypt(shared_secret, encrypted_bytes).map_err(|_| ParserError::UnexpectedError)?;

        // Fr
        let (rem, randomness) = read_scalar(&plaintext_bytes[..])?;

        let (rem, value) = le_u64(rem)?;

        // Memo
        let memo = unsafe { &mut *addr_of_mut!((*out).memo).cast() };
        let rem = Memo::from_bytes_into(rem, memo)?;

        // Asset Identifier
        let asset_id = unsafe { &mut *addr_of_mut!((*out).asset_id).cast() };
        let rem = AssetIdentifier::from_bytes_into(rem, asset_id)?;
        let _asset_id = unsafe { asset_id.assume_init() };

        // PublicAddress
        let sender = unsafe { &mut *addr_of_mut!((*out).sender).cast() };
        let _rem = PublicAddress::from_bytes_into(rem, sender)?;

        unsafe {
            addr_of_mut!((*out).randomness).write(randomness);
            addr_of_mut!((*out).value).write(value);
        }

        Ok(())
    }

    ///// Verify that the note's commitment matches the one passed in
    //pub(crate) fn verify_commitment(&self, commitment: Scalar) -> Result<(), IronfishError> {
    //    if commitment == self.commitment_point() {
    //        Ok(())
    //    } else {
    //        Err(IronfishError::InvalidCommitment)
    //    }
    //}
    //
    ///// Compute the commitment of this note. This is essentially a hash of all
    ///// the note values, including randomness.
    /////
    ///// The owner can publish this value to commit to the fact that the note
    ///// exists, without revealing any of the values on the note until later.
    //pub(crate) fn commitment_point(&self) -> Scalar {
    //    // The commitment is in the prime order subgroup, so mapping the
    //    // commitment to the u-coordinate is an injective encoding.
    //    jubjub::ExtendedPoint::from(self.commitment_full_point())
    //        .to_affine()
    //        .get_u()
    //}
}

impl Note {
    /// Returns the fields to be displayed for this note:
    /// To: destination of the funds
    /// Amount: the amount to be send
    /// asset_id: Only if token is unknown
    pub fn review_fields(
        &self,
        token_list: &TokenList,
    ) -> Result<Vec<(String, String)>, ParserError> {
        use lexical_core::FormattedSize;

        let mut fields = Vec::new();

        zlog_stack("Note::review_fields\n");
        // Format To:
        let to = String::from("To");
        let address = hex::encode(self.owner.public_address());
        fields.push((to, address));

        let mut buffer = [0; u64::FORMATTED_SIZE_DECIMAL + 2];
        let asset_id = hex::encode(self.asset_id.as_bytes());

        // Format amount and asset_id
        if let Some(token) = token_list.token(&asset_id) {
            let amount_label = String::from("Amount");
            // value
            let amount_formatted =
                token_to_fp_str(self.value, &mut buffer[..], token.decimals as usize)?;
            let mut amount_formatted = String::from(
                core::str::from_utf8(amount_formatted).map_err(|_| ParserError::UnexpectedValue)?,
            );
            amount_formatted.push(' ');
            amount_formatted.push_str(token.symbol);

            // push values
            fields.push((amount_label, amount_formatted));
        } else {
            zlog_stack("Note::unknown_token\n");

            #[cfg(feature = "ledger")]
            if !Settings.app_expert_mode() {
                return Err(ParserError::ErrExpertModeMustBeEnabled);
            }

            let amount_label = String::from("Raw Amount ");
            let value_str = u64_to_str(self.value, &mut buffer)?;

            let value_str =
                core::str::from_utf8(value_str).map_err(|_| ParserError::UnexpectedValue)?;

            // push values
            fields.push((amount_label, String::from(value_str)));

            // Add asset_id
            let label = String::from("AssetId ");
            let asset_id = hex::encode(self.asset_id.as_bytes());
            fields.push((label, asset_id));
        }

        Ok(fields)
    }
}
