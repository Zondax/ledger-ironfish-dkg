use blake2b_simd::{Params as Blake2b, State};
use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;

use nom::bytes::complete::take;

use crate::bolos::zlog_stack;
use crate::parser::constants::SPEND_LEN;

use super::FromBytes;
use super::ObjectList;
use crate::parser::ParserError;

// hashing spends:
// pub(crate) fn serialize_signature_fields<W: io::Write>(
//     &self,
//     writer: W,
// ) -> Result<(), IronfishError> {
//     serialize_signature_fields(
//         writer,
//         &self.proof,
//         &self.value_commitment,
//         &self.root_hash,
//         self.tree_size,
//         &self.nullifier,
//     )
// }

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Clone)]
pub struct Spend<'a>(&'a [u8]);

impl<'a> FromBytes<'a> for Spend<'a> {
    #[inline(never)]
    fn from_bytes_into(
        input: &'a [u8],
        out: &mut MaybeUninit<Spend<'a>>,
    ) -> Result<&'a [u8], nom::Err<ParserError>> {
        zlog_stack("Spend::from_bytes_into\n");
        let out = out.as_mut_ptr();
        let (rem, data) = take(SPEND_LEN)(input)?;

        unsafe {
            addr_of_mut!((*out).0).write(data);
        }

        Ok(rem)
    }
}

impl<'a> Spend<'a> {
    #[inline(never)]
    pub fn hash(&self, hasher: &mut State) {
        // both serialization and
        // hashing uses the same serialize_signature_fields
        // function so we can be sure inner data is correctly passed
        // to the hasher
        hasher.update(self.0);
    }
}
