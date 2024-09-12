/// A convenience wrapper around an asset id byte-array, allowing us to push the
/// error checking of the asset id validity to instantiation
/// instead of when trying to get the generator point. This causes code relating
/// to notes and value commitments to be a bit cleaner
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AssetIdentifier([u8; crate::parser::constants::ASSET_ID_LENGTH]);
