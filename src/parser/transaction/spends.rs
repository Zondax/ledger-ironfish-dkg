use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;

use nom::bytes::complete::take;

use crate::parser::constants::SPEND_LEN;

use super::FromBytes;
use super::ObjectList;
use crate::parser::ParserError;

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Clone)]
pub struct Spend<'a>(&'a [u8]);

impl<'a> FromBytes<'a> for Spend<'a> {
    #[inline(never)]
    fn from_bytes_into(
        input: &'a [u8],
        out: &mut MaybeUninit<Spend<'a>>,
    ) -> Result<&'a [u8], nom::Err<ParserError>> {
        let out = out.as_mut_ptr();
        let (rem, data) = take(SPEND_LEN)(input)?;

        unsafe {
            addr_of_mut!((*out).0).write(data);
        }

        Ok(rem)
    }
}
