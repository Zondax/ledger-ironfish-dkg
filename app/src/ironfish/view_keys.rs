/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! View keys allow your transactions to be read
//! by a third party without giving the option to spend your
//! coins. This was designed for auditing systems, but may have other purposes
//! such as in the use of light clients.
//!
//! There are two kinds of view keys. One allows you to share transactions
//! that you have received, while the other allows you to share transactions
//! that you have spent.
//!

use blake2b_simd::Params as Blake2b;
use jubjub::AffinePoint;

use crate::ironfish::public_address::PublicAddress;

const DIFFIE_HELLMAN_PERSONALIZATION: &[u8; 16] = b"Iron Fish shared";

/// Key that allows someone to view a transaction that you have received.
///
/// Referred to as `ivk` in the literature.
#[derive(Clone)]
pub struct IncomingViewKey {
    pub(crate) view_key: [u8; 32],
}

impl IncomingViewKey {
    /// Generate a public address from the incoming viewing key
    pub fn public_address(&self) -> PublicAddress {
        PublicAddress::from_view_key(self)
    }
}
/// Contains two keys that are required (along with outgoing view key)
/// to have full view access to an account.
/// Referred to as `ViewingKey` in the literature.
#[derive(Clone)]
pub struct ViewKey {
    /// Part of the full viewing key. Generally referred to as
    /// `ak` in the literature. Derived from spend_authorizing_key using scalar
    /// multiplication in Sapling. Used to construct incoming viewing key.
    pub authorizing_key: jubjub::AffinePoint,
    /// Part of the full viewing key. Generally referred to as
    /// `nk` in the literature. Derived from proof_authorizing_key using scalar
    /// multiplication. Used to construct incoming viewing key.
    pub nullifier_deriving_key: jubjub::AffinePoint,
}

/// Key that allows someone to view a transaction that you have spent.
///
/// Referred to as `ovk` in the literature.
#[derive(Clone)]
pub struct OutgoingViewKey {
    pub(crate) view_key: [u8; 32],
}

#[cfg(test)]
impl OutgoingViewKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self { view_key: bytes }
    }
}

#[derive(Clone)]
pub struct ProofGenerationKey {
    pub ak: jubjub::AffinePoint,
    pub nsk: jubjub::Fr,
}

/// Derive a shared secret key from a secret key and the other person's public
/// key.
///
/// The shared secret point is calculated by multiplying the public and private
/// keys. This gets converted to bytes and hashed together with the reference
/// public key to generate the final shared secret as used in encryption.

/// A Diffie Hellman key exchange might look like this:
///  *  Alice generates her DH secret key as SaplingKeys::internal_viewing_key
///  *  Alice publishes her Public key
///      *  This becomes her DH public_key
///  *  Bob chooses some randomness as his secret key
///  *  Bob's public key is calculated as (PUBLIC_KEY_GENERATOR * Bob secret key)
///      *  This public key becomes the reference public key for both sides
///      *  Bob sends public key to Alice
///  *  Bob calculates shared secret key as (Alice public key * Bob secret key)
///      *  which is (Alice public key * Bob secret key)
///      *  which is equivalent to (Alice internal viewing key * PUBLIC_KEY_GENERATOR * Bob secret key)
///  *  Alice calculates shared secret key as (Bob public key * Alice internal viewing key)
///      *  which is equivalent to (Alice internal viewing key * PUBLIC_KEY_GENERATOR * Bob secret key)
///  *  both Alice and Bob hash the shared secret key with the reference public
///     key (Bob's public key) to get the final shared secret
///
/// The resulting key can be used in any symmetric cipher
#[must_use]
pub(crate) fn shared_secret(
    secret_key: &jubjub::Fr,
    other_public_key: &AffinePoint, // Previously a SubgroupPoint
    reference_public_key: &AffinePoint,
) -> [u8; 32] {
    let shared_secret = other_public_key * secret_key; // ExtendedPoint
                                                       // Because we are not using ExtendedPoint but AffinePoint, lets convert it
    let affine = AffinePoint::from(&shared_secret).to_bytes();
    hash_shared_secret(&affine, reference_public_key)
}

#[must_use]
fn hash_shared_secret(shared_secret: &[u8; 32], reference_public_key: &AffinePoint) -> [u8; 32] {
    let reference_bytes = reference_public_key.to_bytes();

    let mut hasher = Blake2b::new()
        .hash_length(32)
        .personal(DIFFIE_HELLMAN_PERSONALIZATION)
        .to_state();

    hasher.update(&shared_secret[..]);
    hasher.update(&reference_bytes);

    let mut hash_result = [0; 32];
    hash_result[..].copy_from_slice(hasher.finalize().as_ref());
    hash_result
}

///// Equivalent to calling `shared_secret()` multiple times on the same
///// `other_public_key`/`reference_public_key`, but more efficient.
//#[must_use]
//pub(crate) fn shared_secrets(
//    secret_keys: &[[u8; 32]],
//    other_public_key: &AffinePoint,
//    reference_public_key: &AffinePoint,
//) -> Vec<[u8; 32]> {
//    let shared_secrets = other_public_key.as_extended().multiply_many(secret_keys);
//    shared_secrets
//        .into_iter()
//        .map(move |shared_secret| {
//            hash_shared_secret(&shared_secret.to_bytes(), reference_public_key)
//        })
//        .collect()
//}
