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


#[derive(Clone)]
pub struct ProofGenerationKey {
    pub ak: jubjub::AffinePoint,
    pub nsk: jubjub::Fr,
}