use blake2b_simd::{Params as Blake2b, State};
use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;

use nom::bytes::complete::take;

use crate::bolos::zlog_stack;
use crate::parser::constants::OUTPUT_LEN;
use crate::parser::MerkleNote;

use super::FromBytes;
use super::ObjectList;
use crate::parser::ParserError;

// https://github.com/iron-fish/ironfish/blob/master/ironfish-rust/src/transaction/outputs.rs#L133
// a proof: 192 bytes
// and a merkle note: 328 bytes
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Clone)]
pub struct Output<'a>(&'a [u8]);

impl<'a> FromBytes<'a> for Output<'a> {
    #[inline(never)]
    fn from_bytes_into(
        input: &'a [u8],
        out: &mut MaybeUninit<Output<'a>>,
    ) -> Result<&'a [u8], nom::Err<ParserError>> {
        zlog_stack("Output::from_bytes_into\n");
        // Lazy parsing of output
        // later we can parse each field
        let output = out.as_mut_ptr();
        let (rem, data) = take(OUTPUT_LEN)(input)?;

        unsafe {
            addr_of_mut!((*output).0).write(data);

            // Check that inner fields are parsed correctly
            let out = out.assume_init_ref();
            let mut note = MaybeUninit::uninit();
            out.note_into(&mut note)?;
        }

        Ok(rem)
    }
}

impl<'a> Output<'a> {
    const PROOF_POS: usize = 0;
    const NOTE_POS: usize = 192;

    pub fn raw_note(&self) -> &'a [u8] {
        &self.0[Self::NOTE_POS..]
    }

    pub fn raw_proof(&self) -> &'a [u8] {
        &self.0[..Self::NOTE_POS]
    }

    #[inline(never)]
    pub fn note_into(
        &self,
        out: &mut MaybeUninit<MerkleNote<'a>>,
    ) -> Result<(), nom::Err<ParserError>> {
        MerkleNote::from_bytes_into(self.raw_note(), out)?;
        Ok(())
    }

    #[inline(never)]
    pub fn note(&self) -> Result<MerkleNote<'a>, nom::Err<ParserError>> {
        let mut note = MaybeUninit::uninit();
        self.note_into(&mut note)?;
        unsafe { Ok(note.assume_init()) }
    }

    #[inline(never)]
    pub fn hash(&self, hasher: &mut State) {
        // both serialization and
        // hashing uses the same serialize_signature_fields
        // function so we can be sure inner data is correctly passed
        // to the hasher
        hasher.update(self.0);
    }
}
