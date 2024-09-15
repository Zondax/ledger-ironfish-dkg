use core::fmt::{self, Display, Formatter};
use core::ptr::addr_of_mut;

use crate::crypto::parse_affine_point;
use crate::ironfish::constants::PUBLIC_KEY_GENERATOR;
use crate::ironfish::errors::IronfishError;
use crate::ironfish::sapling::SaplingKey;
use crate::ironfish::view_keys::IncomingViewKey;
use crate::FromBytes;
use alloc::string::String;
use arrayref::array_ref;
use jubjub::AffinePoint;
use nom::bytes::complete::take;

pub const PUBLIC_ADDRESS_SIZE: usize = 32;

/// The address to which funds can be sent, stored as a public
/// transmission key. Using the incoming_viewing_key allows
/// the creation of a unique public addresses without revealing the viewing key.
#[derive(Clone, Copy)]
pub struct PublicAddress(pub(crate) AffinePoint);

impl<'a> FromBytes<'a> for PublicAddress {
    #[inline(never)]
    fn from_bytes_into(
        input: &'a [u8],
        out: &mut core::mem::MaybeUninit<Self>,
    ) -> Result<&'a [u8], nom::Err<crate::parser::ParserError>> {
        let (rem, raw) = take(PUBLIC_ADDRESS_SIZE)(input)?;
        let raw = array_ref![raw, 0, PUBLIC_ADDRESS_SIZE];
        let point = parse_affine_point(raw)?;

        let out = out.as_mut_ptr();

        unsafe {
            addr_of_mut!((*out).0).write(point);
        }

        Ok(rem)
    }
}

impl PublicAddress {
    /// Initialize a public address from its 32 byte representation.
    pub fn new(bytes: &[u8; PUBLIC_ADDRESS_SIZE]) -> Result<Self, IronfishError> {
        Option::from(AffinePoint::from_bytes(*bytes))
            .map(PublicAddress)
            .ok_or_else(|| IronfishError::InvalidPaymentAddress)
    }

    /// Initialize a public address from a sapling key. Typically constructed from
    /// SaplingKey::public_address()
    pub fn from_key(sapling_key: &SaplingKey) -> PublicAddress {
        Self::from_view_key(sapling_key.incoming_view_key())
    }

    pub fn from_view_key(view_key: &IncomingViewKey) -> PublicAddress {
        let extended_point = PUBLIC_KEY_GENERATOR.multiply_bits(&view_key.view_key);
        let result = AffinePoint::from(&extended_point);
        PublicAddress(result)
    }

    /// Retrieve the public address in byte form.
    pub fn public_address(&self) -> [u8; PUBLIC_ADDRESS_SIZE] {
        self.0.to_bytes()
    }
}

// This is used for formatting the pub address
// during UI
impl Display for PublicAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Reference implementation just compute hex
        // representation of 32-byte addresses
        write!(f, "{}", hex::encode(self.public_address()))
    }
}
