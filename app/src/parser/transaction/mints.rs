use blake2b_simd::{Params as Blake2b, State};
use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;

use nom::bytes::complete::take;

use crate::bolos::zlog_stack;
use crate::parser::constants::MINT_LEN;

use super::FromBytes;
use super::ObjectList;
use crate::parser::ParserError;

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Clone)]
pub struct Mint<'a>(&'a [u8]);

impl<'a> FromBytes<'a> for Mint<'a> {
    #[inline(never)]
    fn from_bytes_into(
        input: &'a [u8],
        out: &mut MaybeUninit<Mint<'a>>,
    ) -> Result<&'a [u8], nom::Err<ParserError>> {
        zlog_stack("Mint::from_bytes_into\n");
        let out = out.as_mut_ptr();
        let (rem, data) = take(MINT_LEN)(input)?;

        unsafe {
            addr_of_mut!((*out).0).write(data);
        }

        Ok(rem)
    }
}

//
// pub fn write<W: io::Write>(
//     &self,
//     mut writer: W,
//     version: TransactionVersion,
// ) -> Result<(), IronfishError> {
//     writer.write_all(&self.public_key_randomness.to_bytes())?;
//     self.description.write(&mut writer, version)?;
//
//     Ok(())
// }
//
// MintDescription::write
// pub fn write<W: io::Write>(
//     &self,
//     mut writer: W,
//     version: TransactionVersion,
// ) -> Result<(), IronfishError> {
//     self.serialize_signature_fields(&mut writer, version)?;
//     self.authorizing_signature.write(&mut writer)?;
//
//     Ok(())
// }

/// Stow the bytes of this [`MintDescription`] in the given writer.
/// Write the signature of this proof to the provided writer.
///
/// The signature is used by the transaction to calculate the signature
/// hash. Having this data essentially binds the note to the transaction,
/// proving that it is actually part of that transaction.

impl<'a> Mint<'a> {
    #[inline(never)]
    pub fn hash(&self, hasher: &mut State) {
        // both serialization and
        // hashing uses the same serialize_signature_fields
        // function so we can be sure inner data is correctly passed
        // to the hasher
        hasher.update(self.0);
    }
}
