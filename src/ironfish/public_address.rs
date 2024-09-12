
use jubjub::AffinePoint;
use crate::ironfish::constants::PUBLIC_KEY_GENERATOR;
use crate::ironfish::errors::IronfishError;
use crate::ironfish::sapling::SaplingKey;
use crate::ironfish::view_keys::IncomingViewKey;

pub const PUBLIC_ADDRESS_SIZE: usize = 32;

/// The address to which funds can be sent, stored as a public
/// transmission key. Using the incoming_viewing_key allows
/// the creation of a unique public addresses without revealing the viewing key.
#[derive(Clone, Copy)]
pub struct PublicAddress(pub(crate) AffinePoint);

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