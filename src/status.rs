use ledger_device_sdk::io::{Reply, StatusWords};

// Application status words.
#[repr(u16)]
#[derive(Clone, Copy, PartialEq)]
pub enum AppSW {
    Deny = 0x6985,
    WrongP1P2 = 0x6A86,
    InsNotSupported = 0x6D00,
    ClaNotSupported = 0x6E00,
    TxDisplayFail = 0xB001,
    AddrDisplayFail = 0xB002,
    TxWrongLength = 0xB004,
    TxParsingFail = 0xB005,
    TxHashFail = 0xB006,
    TxSignFail = 0xB008,
    KeyDeriveFail = 0xB009,
    VersionParsingFail = 0xB00A,
    DkgRound2Fail = 0xB00B,
    DkgRound3Fail = 0xB00C,
    InvalidKeyType = 0xB00D,
    InvalidIdentity = 0xB00E,
    InvalidPayload = 0xB00F,
    BufferOutOfBounds = 0xB010,
    InvalidSigningPackage = 0xB011,
    InvalidRandomizer = 0xB012,
    InvalidSigningNonces = 0xB013,
    InvalidIdentityIndex = 0xB014,
    InvalidKeyPackage = 0xB015,
    InvalidPublicPackage = 0xB016,
    InvalidGroupSecretKey = 0xB017,
    WrongApduLength = StatusWords::BadLen as u16,
    Ok = 0x9000,
}

impl From<AppSW> for Reply {
    fn from(sw: AppSW) -> Reply {
        Reply(sw as u16)
    }
}
