use core::ptr::addr_of_mut;

use alloc::string::String;
use arrayref::array_ref;
use nom::bytes::complete::take;

use crate::{parser::constants::MEMO_SIZE, utils::str_to_array, FromBytes};

/// Memo field on a Note. Used to encode transaction IDs or other information
/// about the transaction.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Memo(pub [u8; MEMO_SIZE]);

impl<'a> FromBytes<'a> for Memo {
    #[inline(never)]
    fn from_bytes_into(
        input: &'a [u8],
        out: &mut core::mem::MaybeUninit<Self>,
    ) -> Result<&'a [u8], nom::Err<crate::parser::ParserError>> {
        let (rem, raw) = take(MEMO_SIZE)(input)?;
        let bytes = array_ref!(raw, 0, MEMO_SIZE);

        let out = out.as_mut_ptr();

        unsafe {
            addr_of_mut!((*out).0).write(*bytes);
        }

        Ok(rem)
    }
}

impl From<&str> for Memo {
    fn from(string: &str) -> Self {
        let memo_bytes = str_to_array(string);
        Memo(memo_bytes)
    }
}

impl From<String> for Memo {
    fn from(string: String) -> Self {
        Memo::from(string.as_str())
    }
}

impl From<[u8; MEMO_SIZE]> for Memo {
    fn from(value: [u8; MEMO_SIZE]) -> Self {
        Memo(value)
    }
}
