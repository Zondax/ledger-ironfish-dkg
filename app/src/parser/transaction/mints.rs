use blake2b_simd::State;
use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;

use nom::bytes::complete::take;

use crate::parser::constants::MINT_LEN;
use crate::TransactionVersion;

use crate::parser::ParserError;

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Clone)]
pub struct MintList<'a> {
    data: &'a [u8],
    version: TransactionVersion,
}

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Clone)]
pub struct Mint<'a> {
    data: &'a [u8],
    has_transfer_ownership_to: bool,
}

#[cfg_attr(test, derive(Debug))]
pub struct MintIterator<'a> {
    data: &'a [u8],
    version: TransactionVersion,
    index: usize,
}

impl<'a> MintList<'a> {
    pub fn parse_into(
        input: &'a [u8],
        version: TransactionVersion,
        num_mints: usize,
        out: &mut MaybeUninit<MintList<'a>>,
    ) -> Result<&'a [u8], nom::Err<ParserError>> {
        let mut total_len = 0;
        let mut remaining = input;

        let mut mint = MaybeUninit::uninit();
        for _ in 0..num_mints {
            let rem = Mint::parse_into(remaining, version, &mut mint)?;
            let obj_ptr = mint.as_mut_ptr();
            unsafe {
                if !version.has_mint_transfer_ownership_to() && (*obj_ptr).has_transfer_ownership_to
                {
                    return Err(ParserError::InvalidMint.into());
                }

                total_len += (*obj_ptr).data.len();

                obj_ptr.drop_in_place();
            }
            remaining = rem;
        }

        let (rem, data) = take(total_len)(input)?;

        let out_ptr = out.as_mut_ptr();
        unsafe {
            addr_of_mut!((*out_ptr).data).write(data);
            addr_of_mut!((*out_ptr).version).write(version);
        }

        Ok(rem)
    }

    pub fn iter(&self) -> MintIterator<'a> {
        MintIterator {
            data: self.data,
            version: self.version,
            index: 0,
        }
    }
}

impl<'a> Mint<'a> {
    #[inline(never)]
    pub fn parse_into(
        input: &'a [u8],
        version: TransactionVersion,
        out: &mut MaybeUninit<Mint<'a>>,
    ) -> Result<&'a [u8], nom::Err<ParserError>> {
        const OWNER_SIZE: usize = 32;
        const FLAG_SIZE: usize = 1;
        const TRANSFER_OWNERSHIP_SIZE: usize = 32;
        const AUTH_SIG_SIZE: usize = 64;

        // Start with the base length (public key randomness is included in MINT_LEN)
        let mut total_len = MINT_LEN;

        // Check if we have at least the base data
        if input.len() < total_len {
            return Err(nom::Err::Failure(ParserError::UnexpectedBufferEnd));
        }

        let mut has_transfer_to = false;

        // If version has mint transfer ownership,
        // we need to add owner size and check for the flag, this increases
        // the amount of bytes to read:
        // 32_bytes owner + 1-byte optional flag(0/1)
        // if that flag is set we read 32_bytes new owner
        if version.has_mint_transfer_ownership_to() {
            total_len += OWNER_SIZE;

            // Check if we have enough data for the owner and flag
            if input.len() < total_len {
                return Err(nom::Err::Failure(ParserError::UnexpectedBufferEnd));
            }

            has_transfer_to = input[total_len] == 1;
            total_len += FLAG_SIZE;

            if has_transfer_to {
                total_len += TRANSFER_OWNERSHIP_SIZE;
            }
        }

        // Add authorizing signature length
        total_len += AUTH_SIG_SIZE;

        // Final check if we have all the data we need
        if input.len() < total_len {
            return Err(nom::Err::Failure(ParserError::UnexpectedBufferEnd));
        }

        let (rem, data) = take(total_len)(input)?;
        let out_ptr = out.as_mut_ptr();
        unsafe {
            addr_of_mut!((*out_ptr).data).write(data);
            addr_of_mut!((*out_ptr).has_transfer_ownership_to).write(has_transfer_to);
        }

        Ok(rem)
    }

    #[inline(never)]
    pub fn run_hash(&self, hasher: &mut State) {
        // both serialization and
        // hashing uses the same serialize_signature_fields
        // function so we can be sure inner data is correctly passed
        // to the hasher, but we need to exclude the latest authorizing_signature
        // bytes(64)
        // and the initial 32-bytes corresponding to the pubkey randomness
        let to_hash = &self.data[32..self.data.len() - 64];
        hasher.update(to_hash);
    }
}

impl<'a> Iterator for MintIterator<'a> {
    type Item = Mint<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() {
            return None;
        }

        let mut mint = MaybeUninit::uninit();
        match Mint::parse_into(self.data, self.version, &mut mint) {
            Ok(rem) => {
                let mint = unsafe { mint.assume_init() };
                self.data = rem;
                self.index += 1;
                Some(mint)
            }
            Err(_) => None,
        }
    }
}
