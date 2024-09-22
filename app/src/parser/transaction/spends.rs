use blake2b_simd::{Params as Blake2b, State};
use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;

use nom::bytes::complete::take;

use crate::bolos::zlog_stack;
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
        zlog_stack("Spend::from_bytes_into\0");
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
        const PUBLIC_KEY_RANDOMNESS_LEN: usize = 32;
        const AUTHORIZING_SIGNATURE_LEN: usize = 64;

        let start = PUBLIC_KEY_RANDOMNESS_LEN;
        let end = self.0.len() - AUTHORIZING_SIGNATURE_LEN;

        // Extract and hash only the relevant parts:
        // - proof (192 bytes)
        // - value_commitment (32 bytes)
        // - root_hash (32 bytes)
        // - tree_size (4 bytes)
        // - nullifier (32 bytes)
        hasher.update(&self.0[start..end]);
    }
}
