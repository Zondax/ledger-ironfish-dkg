use crate::chacha20poly::constants::KEY_LEN;
use alloc::vec;
use alloc::vec::Vec;
use ledger_device_sdk::ecc::{bip32_derive, ChainCode, CurvesId, Secret};

#[inline(never)]
pub fn compute_key() -> [u8; KEY_LEN] {
    let path_0: Vec<u32> = vec![
        (0x80000000 | 0x2c),
        (0x80000000 | 0x53a),
        (0x80000000 | 0x0),
        (0x80000000 | 0x0),
        (0x80000000 | 0x0),
    ];

    let mut secret_key_0 = Secret::<64>::new();
    let mut cc: ChainCode = Default::default();

    // Ignoring 'Result' here because known to be valid
    let _ = bip32_derive(
        CurvesId::Ed25519,
        &path_0,
        secret_key_0.as_mut(),
        Some(cc.value.as_mut()),
    );

    secret_key_0.as_ref()[0..KEY_LEN].try_into().unwrap()
}
