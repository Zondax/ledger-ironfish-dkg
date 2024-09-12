use crate::{ironfish::public_address::PublicAddress, parser::AssetIdentifier};

use super::memo::Memo;

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
