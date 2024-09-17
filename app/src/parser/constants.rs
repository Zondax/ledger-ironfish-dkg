use jubjub::{AffineNielsPoint, AffinePoint, Fq};
// Value commitment
// root of the tree
// tree_size
// nullifier
// Authorizing signature
//

pub const PUBLIC_ADDRESS_SIZE: usize = 32;
pub const SPEND_LEN: usize = 32 + 192 + 32 + 32 + 4 + 32 + 64;
// 192-bytes proof + 328-bytes Merkle Note
pub const OUTPUT_LEN: usize = 192 + 328;
pub const MINT_LEN: usize = 32 + 192 + 193 + 8;
pub const BURN_LEN: usize = 32 + 8;
pub const REDJUBJUB_SIGNATURE_LEN: usize = 64;
pub const KEY_LENGTH: usize = 32;
pub const SCALAR_SIZE: usize = 32;
pub const MEMO_SIZE: usize = 32;
pub const AMOUNT_VALUE_SIZE: usize = 8;

pub const PLAINTEXT_NOTE_SIZE: usize = PUBLIC_ADDRESS_SIZE
    + ASSET_ID_LENGTH
    + AMOUNT_VALUE_SIZE
    + SCALAR_SIZE
    + MEMO_SIZE
    + PUBLIC_ADDRESS_SIZE;

/// Length in bytes of the asset identifier
pub const ASSET_ID_LENGTH: usize = 32;

pub const MAC_SIZE: usize = 16;
// Size of a merkle note
// https://github.com/iron-fish/ironfish/blob/master/ironfish-rust/src/note.rs#L30
pub const ENCRYPTED_NOTE_SIZE: usize =
    SCALAR_SIZE + MEMO_SIZE + AMOUNT_VALUE_SIZE + ASSET_ID_LENGTH + PUBLIC_ADDRESS_SIZE;

pub const ENCRYPTED_SHARED_KEY_SIZE: usize = 64;

pub const NOTE_ENCRYPTION_KEY_SIZE: usize = ENCRYPTED_SHARED_KEY_SIZE + MAC_SIZE;

pub const AFFINE_POINT_SIZE: usize = 32;

/// BLAKE2s personalization for deriving asset identifier from asset name
pub const ASSET_ID_PERSONALIZATION: &[u8; 8] = b"ironf_A_";

/// BLAKE2s personalization for PRF^nf = BLAKE2s(nk | rho)
pub const PRF_NF_PERSONALIZATION: &[u8; 8] = b"ironf_nf";

/// BLAKE2s personalization for the value commitment generator for the value
pub const VALUE_COMMITMENT_GENERATOR_PERSONALIZATION: &[u8; 8] = b"ironf_cv";