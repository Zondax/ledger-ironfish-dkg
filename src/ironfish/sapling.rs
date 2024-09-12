use crate::ironfish::constants::{CRH_IVK_PERSONALIZATION, PROOF_GENERATION_KEY_GENERATOR, SPENDING_KEY_GENERATOR};
use blake2b_simd::Params as Blake2b;
use blake2s_simd::Params as Blake2s;
use jubjub::{AffinePoint};
use crate::ironfish::errors::IronfishError;
use crate::ironfish::view_keys::{IncomingViewKey, OutgoingViewKey, ProofGenerationKey, ViewKey};

const EXPANDED_SPEND_BLAKE2_KEY: &[u8; 16] = b"Iron Fish Money ";

pub const SPEND_KEY_SIZE: usize = 32;

/// A single private key generates multiple other key parts that can
/// be used to allow various forms of access to a commitment note:
///
/// While the key parts are all represented as 256 bit keys to the outside
/// world, inside the API they map to Edwards points or scalar values
/// on the JubJub curve.
#[derive(Clone)]
pub struct SaplingKey {
    /// The private (secret) key from which all the other key parts are derived.
    /// The expanded form of this key is required before a note can be spent.
    spending_key: [u8; SPEND_KEY_SIZE],

    /// Part of the expanded form of the spending key, generally referred to as
    /// `ask` in the literature. Derived from spending key using a seeded
    /// pseudorandom hash function. Used to construct authorizing_key.
    pub(crate) spend_authorizing_key: jubjub::Fr,

    /// Part of the expanded form of the spending key, generally referred to as
    /// `nsk` in the literature. Derived from spending key using a seeded
    /// pseudorandom hash function. Used to construct nullifier_deriving_key
    pub(crate) proof_authorizing_key: jubjub::Fr,

    /// Part of the expanded form of the spending key, as well as being used
    /// directly in the full viewing key. Generally referred to as
    /// `ovk` in the literature. Derived from spending key using a seeded
    /// pseudorandom hash function. This allows the creator of a note to access
    /// keys needed to decrypt the note's contents.
    pub(crate) outgoing_viewing_key: OutgoingViewKey,

    /// Part of the full viewing key. Contains ak/nk from literature, used for deriving nullifiers
    /// and therefore spends
    pub(crate) view_key: ViewKey,

    /// Part of the payment_address. Generally referred to as
    /// `ivk` in the literature. Derived from authorizing key and
    /// nullifier deriving key. Used to construct payment address and
    /// transmission key. This key allows the receiver of a note to decrypt its
    /// contents. Derived from view_key contents, this is materialized for convenience
    pub(crate) incoming_viewing_key: IncomingViewKey,
}

impl SaplingKey {
    /// Construct a new key from an array of bytes
    pub fn new(spending_key: [u8; SPEND_KEY_SIZE]) -> Result<Self, IronfishError> {
        // ask
        let spend_authorizing_key =
            jubjub::Fr::from_bytes_wide(&Self::convert_key(spending_key, 0));

        if spend_authorizing_key == jubjub::Fr::zero() {
            return Err(IronfishError::IllegalValue);
        }

        // nsk
        let proof_authorizing_key =
            jubjub::Fr::from_bytes_wide(&Self::convert_key(spending_key, 1));

        // ovk
        let mut outgoing_viewing_key = [0; SPEND_KEY_SIZE];
        outgoing_viewing_key[0..SPEND_KEY_SIZE]
            .clone_from_slice(&Self::convert_key(spending_key, 2)[0..SPEND_KEY_SIZE]);
        let outgoing_viewing_key = OutgoingViewKey {
            view_key: outgoing_viewing_key,
        };
        // ak
        let authorizing_key = AffinePoint::from(SPENDING_KEY_GENERATOR.multiply_bits(&spend_authorizing_key.to_bytes()));
        //nk
        let nullifier_deriving_key = AffinePoint::from(PROOF_GENERATION_KEY_GENERATOR.multiply_bits(&proof_authorizing_key.to_bytes()));
        let view_key = ViewKey {
            authorizing_key,
            nullifier_deriving_key,
        };
        // ivk
        let incoming_viewing_key = IncomingViewKey {
            view_key: Self::hash_viewing_key(&authorizing_key, &nullifier_deriving_key)?,
        };

        Ok(SaplingKey {
            spending_key,
            spend_authorizing_key,
            proof_authorizing_key,
            outgoing_viewing_key,
            view_key,
            incoming_viewing_key,
        })
    }

    /// Convert the spending key to another value using a pseudorandom hash
    /// function. Used during key construction to derive the following keys:
    ///  *  `spend_authorizing_key` (represents a sapling scalar Fs type)
    ///  *  `proof_authorizing_key` (represents a sapling scalar Fs type)
    ///  *  `outgoing_viewing_key (just some bytes)
    ///
    /// # Arguments
    ///  *  `spending_key` The 32 byte spending key
    ///  *  `modifier` a byte to add to tweak the hash for each of the three
    ///     values
    fn convert_key(spending_key: [u8; SPEND_KEY_SIZE], modifier: u8) -> [u8; 64] {
        let mut hasher = Blake2b::new()
            .hash_length(64)
            .personal(EXPANDED_SPEND_BLAKE2_KEY)
            .to_state();

        hasher.update(&spending_key);
        hasher.update(&[modifier]);
        let mut hash_result = [0; 64];
        hash_result[0..64].clone_from_slice(&hasher.finalize().as_ref()[0..64]);
        hash_result
    }

    /// Helper method to construct the viewing key from the authorizing key
    /// and nullifier deriving key using a blake2 hash of their respective bytes.
    ///
    /// This method is only called once, but it's kind of messy, so I pulled it
    /// out of the constructor for easier maintenance.
    pub fn hash_viewing_key(
        authorizing_key: &AffinePoint,
        nullifier_deriving_key: &AffinePoint,
    ) -> Result<[u8; 32], IronfishError> {
        let mut view_key_contents = [0; 64];
        view_key_contents[0..32].copy_from_slice(&authorizing_key.to_bytes());
        view_key_contents[32..64].copy_from_slice(&nullifier_deriving_key.to_bytes());
        // let mut hasher = Blake2s::with_params(32, &[], &[], CRH_IVK_PERSONALIZATION);

        let mut hash_result = [0; 32];
        hash_result.copy_from_slice(
            Blake2s::new()
                .hash_length(32)
                .personal(CRH_IVK_PERSONALIZATION)
                .hash(&view_key_contents)
                .as_bytes(),
        );
        // Drop the last five bits, so it can be interpreted as a scalar.
        hash_result[31] &= 0b0000_0111;
        if hash_result == [0; 32] {
            return Err(IronfishError::InvalidViewingKey);
        }

        Ok(hash_result)
    }

    /// Retrieve the publicly visible outgoing viewing key
    pub fn outgoing_view_key(&self) -> &OutgoingViewKey {
        &self.outgoing_viewing_key
    }

    /// Retrieve the publicly visible incoming viewing key
    pub fn incoming_view_key(&self) -> &IncomingViewKey {
        &self.incoming_viewing_key
    }

    /// Adapter to convert this key to a proof generation key for use in
    /// sapling functions
    pub fn sapling_proof_generation_key(&self) -> ProofGenerationKey {
        ProofGenerationKey {
            ak: self.view_key.authorizing_key,
            nsk: self.proof_authorizing_key,
        }
    }
}