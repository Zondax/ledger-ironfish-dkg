use core::ptr::addr_of_mut;

use arrayref::array_ref;
use nom::bytes::complete::take;

use crate::FromBytes;

use crate::parser::constants::ASSET_ID_LENGTH;

/// A convenience wrapper around an asset id byte-array, allowing us to push the
/// error checking of the asset id validity to instantiation
/// instead of when trying to get the generator point. This causes code relating
/// to notes and value commitments to be a bit cleaner
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AssetIdentifier([u8; ASSET_ID_LENGTH]);

impl AssetIdentifier {
    pub fn as_bytes(&self) -> &[u8; ASSET_ID_LENGTH] {
        &self.0
    }
}

impl<'a> FromBytes<'a> for AssetIdentifier {
    #[inline(never)]
    fn from_bytes_into(
        input: &'a [u8],
        out: &mut core::mem::MaybeUninit<Self>,
    ) -> Result<&'a [u8], nom::Err<crate::parser::ParserError>> {
        let (rem, raw) = take(ASSET_ID_LENGTH)(input)?;
        let bytes = array_ref!(raw, 0, ASSET_ID_LENGTH);

        let out = out.as_mut_ptr();

        unsafe {
            addr_of_mut!((*out).0).write(*bytes);
        }

        Ok(rem)
    }
}
