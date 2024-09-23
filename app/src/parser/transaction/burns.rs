use blake2b_simd::{Params as Blake2b, State};
use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;

use nom::bytes::complete::take;

use crate::bolos::zlog_stack;
use crate::parser::constants::BURN_LEN;

use super::FromBytes;
use super::ObjectList;
use crate::parser::ParserError;

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Clone)]
pub struct Burn<'a>(&'a [u8]);

impl<'a> FromBytes<'a> for Burn<'a> {
    #[inline(never)]
    fn from_bytes_into(
        input: &'a [u8],
        out: &mut MaybeUninit<Burn<'a>>,
    ) -> Result<&'a [u8], nom::Err<ParserError>> {
        zlog_stack("Burn::from_bytes_into\n");
        let out = out.as_mut_ptr();

        let (rem, data) = take(BURN_LEN)(input)?;

        unsafe {
            addr_of_mut!((*out).0).write(data);
        }

        Ok(rem)
    }
}

impl<'a> Burn<'a> {
    #[inline(never)]
    pub fn hash(&self, hasher: &mut State) {
        // both serialization and
        // hashing uses the same serialize_signature_fields
        // function so we can be sure inner data is correctly passed
        // to the hasher
        hasher.update(self.0);
    }
}
