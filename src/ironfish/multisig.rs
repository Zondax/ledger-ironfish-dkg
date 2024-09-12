use jubjub::{AffinePoint, Fr};
use crate::ironfish::constants::PROOF_GENERATION_KEY_GENERATOR;
use crate::ironfish::public_address::PublicAddress;
use crate::ironfish::sapling::SaplingKey;
use crate::ironfish::view_keys::{IncomingViewKey, OutgoingViewKey, ViewKey};

pub struct MultisigAccountKeys {
    /// Equivalent to [`crate::keys::SaplingKey::proof_authorizing_key`]
    pub proof_authorizing_key: jubjub::Fr,
    /// Equivalent to [`crate::keys::SaplingKey::outgoing_viewing_key`]
    pub outgoing_viewing_key: OutgoingViewKey,
    /// Equivalent to [`crate::keys::SaplingKey::view_key`]
    pub view_key: ViewKey,
    /// Equivalent to [`crate::keys::SaplingKey::incoming_viewing_key`]
    pub incoming_viewing_key: IncomingViewKey,
    /// Equivalent to [`crate::keys::SaplingKey::public_address`]
    pub public_address: PublicAddress,
}

pub fn  derive_account_keys(
    authorizing_key: &[u8; 32], //&VerifyingKey,
    group_secret_key: &[u8; 32],
) -> MultisigAccountKeys {
    // Group secret key (gsk), obtained from the multisig setup process
    let group_secret_key =
        SaplingKey::new(*group_secret_key).expect("failed to derive group secret key");

    // Authorization key (ak), obtained from the multisig setup process
    let authorizing_key = Option::from(AffinePoint::from_bytes(*authorizing_key))
        .expect("failied to derive authorizing key");

    // Nullifier keys (nsk and nk), derived from the gsk
    let proof_authorizing_key = Fr::from(group_secret_key.sapling_proof_generation_key().nsk);
    let nullifier_deriving_key_ep = PROOF_GENERATION_KEY_GENERATOR.multiply_bits(&proof_authorizing_key.to_bytes());
    let nullifier_deriving_key = AffinePoint::from(&nullifier_deriving_key_ep);

    // Incoming view key (ivk), derived from the ak and the nk
    let view_key = ViewKey {
        authorizing_key,
        nullifier_deriving_key,
    };
    let incoming_viewing_key = IncomingViewKey {
        view_key: SaplingKey::hash_viewing_key(&authorizing_key, &nullifier_deriving_key)
            .expect("failed to derive view key"),
    };

    // Outgoing view key (ovk), derived from the gsk
    let outgoing_viewing_key = group_secret_key.outgoing_view_key().clone();

    // Public address (pk), derived from the ivk
    let public_address = incoming_viewing_key.public_address();

    MultisigAccountKeys {
        proof_authorizing_key,
        outgoing_viewing_key,
        view_key,
        incoming_viewing_key,
        public_address,
    }
}
