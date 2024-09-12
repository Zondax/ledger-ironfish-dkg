use core::{mem::MaybeUninit, ptr::addr_of_mut};

use nom::{
    bytes::complete::take,
    number::complete::{le_i64, le_u32, le_u64, le_u8},
};

use crate::parser::constants::{KEY_LENGTH, REDJUBJUB_SIGNATURE_LEN};

mod burns;
mod mints;
mod outputs;
mod spends;

use super::{FromBytes, ObjectList, TransactionVersion};
pub use burns::Burn;
pub use mints::Mint;
pub use outputs::Output;
pub use spends::Spend;

// parser_error_t _read(parser_context_t *ctx, parser_tx_t *v) {
//     CHECK_ERROR(readTransactionVersion(ctx, &v->transactionVersion));
//     CHECK_ERROR(readUint64(ctx, &v->spends.elements));
//     CHECK_ERROR(readUint64(ctx, &v->outputs.elements));
//     CHECK_ERROR(readUint64(ctx, &v->mints.elements));
//     CHECK_ERROR(readUint64(ctx, &v->burns.elements));
//     CHECK_ERROR(readInt64(ctx, &v->fee));
//     CHECK_ERROR(readUint32(ctx, &v->expiration));
//
//     v->randomizedPublicKey.len = KEY_LENGTH;
//     CHECK_ERROR(readBytes(ctx, &v->randomizedPublicKey.ptr, v->randomizedPublicKey.len));
//
//     v->publicKeyRandomness.len = KEY_LENGTH;
//     CHECK_ERROR(readBytes(ctx, &v->publicKeyRandomness.ptr, v->publicKeyRandomness.len));
//
//     // Read Spends and Outputs
//     CHECK_ERROR(readSpends(ctx, &v->spends));
//     CHECK_ERROR(readOutputs(ctx, &v->outputs));
//
//     // Read Mints and Burns
//     CHECK_ERROR(readMints(ctx, &v->mints, v->transactionVersion));
//     CHECK_ERROR(readBurns(ctx, &v->burns));
//
//     v->bindingSignature.len = REDJUBJUB_SIGNATURE_LEN;
//     CHECK_ERROR(readBytes(ctx, &v->bindingSignature.ptr, v->bindingSignature.len));
//
//     if (ctx->bufferLen != ctx->offset) {
//         return parser_unexpected_buffer_end;
//     }
//
//     CHECK_ERROR(transaction_signature_hash(v, v->transactionHash));
//     return parser_ok;
// }

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Clone)]
pub struct Transaction<'a> {
    tx_version: TransactionVersion,
    random_pubkey: &'a [u8; KEY_LENGTH],
    pubkey_randomness: &'a [u8; KEY_LENGTH],

    spends: ObjectList<'a, Spend<'a>>,
    outputs: ObjectList<'a, Output<'a>>,
    mints: ObjectList<'a, Mint<'a>>,
    burns: ObjectList<'a, Burn<'a>>,
    fee: i64,
    expiration: u32,
    binding_sig: &'a [u8; REDJUBJUB_SIGNATURE_LEN],
}

impl<'a> FromBytes<'a> for Transaction<'a> {
    #[inline(never)]
    fn from_bytes_into(
        input: &'a [u8],
        out: &mut core::mem::MaybeUninit<Self>,
    ) -> Result<&'a [u8], nom::Err<super::ParserError>> {
        let out = out.as_mut_ptr();

        let (rem, raw_version) = le_u8(input)?;
        let version = TransactionVersion::try_from(raw_version)?;
        // now read the number of spends, outputs, mints and burns
        let (rem, num_spends) = le_u64(rem)?;
        let (rem, num_outputs) = le_u64(rem)?;
        let (rem, num_mints) = le_u64(rem)?;
        let (rem, num_burns) = le_u64(rem)?;
        // now read the fee and expiration
        let (rem, fee) = le_i64(rem)?;
        let (rem, expiration) = le_u32(rem)?;

        // This fields bellows are present in C parser, we need to figure out where to
        // place this information
        // rondomizedPublicKey
        let (rem, random_pubkey) = take(KEY_LENGTH)(rem)?;
        // publicKeyRandomness
        let (rem, randomness) = take(KEY_LENGTH)(rem)?;

        let random_pubkey = arrayref::array_ref![random_pubkey, 0, KEY_LENGTH];
        let pubkey_randomness = arrayref::array_ref![randomness, 0, KEY_LENGTH];

        let spends: &mut MaybeUninit<ObjectList<'a, Spend<'a>>> =
            unsafe { &mut *addr_of_mut!((*out).spends).cast() };
        let rem = ObjectList::new_into_with_len(rem, spends, num_spends as usize)?;

        let outputs: &mut MaybeUninit<ObjectList<'a, Output<'a>>> =
            unsafe { &mut *addr_of_mut!((*out).outputs).cast() };
        let rem = ObjectList::new_into_with_len(rem, outputs, num_outputs as usize)?;

        let mints: &mut MaybeUninit<ObjectList<'a, Mint<'a>>> =
            unsafe { &mut *addr_of_mut!((*out).mints).cast() };
        let rem = ObjectList::new_into_with_len(rem, mints, num_mints as usize)?;

        let burns: &mut MaybeUninit<ObjectList<'a, Burn<'a>>> =
            unsafe { &mut *addr_of_mut!((*out).burns).cast() };
        let rem = ObjectList::new_into_with_len(rem, burns, num_burns as usize)?;

        let (rem, sig) = take(REDJUBJUB_SIGNATURE_LEN)(rem)?;
        let binding_sig = arrayref::array_ref![sig, 0, REDJUBJUB_SIGNATURE_LEN];

        unsafe {
            addr_of_mut!((*out).tx_version).write(version);
            addr_of_mut!((*out).fee).write(fee);
            addr_of_mut!((*out).expiration).write(expiration);
            addr_of_mut!((*out).binding_sig).write(binding_sig);
            addr_of_mut!((*out).random_pubkey).write(random_pubkey);
            addr_of_mut!((*out).pubkey_randomness).write(pubkey_randomness);
        }

        Ok(input)
    }
}

impl<'a> Transaction<'a> {
    pub fn num_spends(&self) -> usize {
        self.spends.iter().count()
    }

    pub fn num_outputs(&self) -> usize {
        self.outputs.iter().count()
    }

    pub fn num_mints(&self) -> usize {
        self.mints.iter().count()
    }

    pub fn num_burns(&self) -> usize {
        self.burns.iter().count()
    }
}
