use ledger_device_sdk::io::{Reply, StatusWords};

use crate::{ironfish::errors::IronfishError, parser::ParserError};

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
    InvalidScalar = 0xB018,
    DecryptionFail = 0xB019,
    EncryptionFail = 0xB020,
    InvalidNVMWrite = 0xB021,
    WrongApduLength = StatusWords::BadLen as u16,
    Ok = 0x9000,
}

impl From<IronfishError> for AppSW {
    fn from(error: IronfishError) -> Self {
        match error {
            IronfishError::InvalidRandomizer => AppSW::InvalidRandomizer,
            IronfishError::InvalidSignature => AppSW::TxSignFail,
            IronfishError::InvalidPublicAddress => AppSW::AddrDisplayFail,
            IronfishError::InvalidTransaction => AppSW::TxParsingFail,
            IronfishError::InvalidTransactionVersion => AppSW::VersionParsingFail,
            IronfishError::InvalidPaymentAddress => AppSW::AddrDisplayFail,
            IronfishError::InvalidData => AppSW::InvalidPayload,
            IronfishError::RoundTwoSigningFailure => AppSW::DkgRound2Fail,
            IronfishError::InvalidSigningKey => AppSW::KeyDeriveFail,
            IronfishError::InvalidSecret => AppSW::InvalidGroupSecretKey,
            // For errors that don't have a direct mapping, use a generic error
            _ => AppSW::Deny,
        }
    }
}

impl From<ParserError> for AppSW {
    fn from(error: ParserError) -> Self {
        match error {
            ParserError::Ok => AppSW::Ok,
            ParserError::UnexpectedBufferEnd => AppSW::BufferOutOfBounds,
            ParserError::ValueOutOfRange => AppSW::TxParsingFail,
            ParserError::OperationOverflows => AppSW::TxParsingFail,
            ParserError::UnexpectedValue => AppSW::TxParsingFail,
            ParserError::UnexpectedType => AppSW::TxParsingFail,
            ParserError::InvalidTxVersion => AppSW::VersionParsingFail,
            ParserError::InvalidKey => AppSW::InvalidKeyPackage,
            ParserError::InvalidAffinePoint => AppSW::InvalidPublicPackage,
            ParserError::InvalidTypeId => AppSW::TxParsingFail,
            ParserError::InvalidSpend => AppSW::TxParsingFail,
            ParserError::InvalidOuptut => AppSW::TxParsingFail,
            ParserError::InvalidMint => AppSW::TxParsingFail,
            ParserError::InvalidBurn => AppSW::TxParsingFail,
            ParserError::UnexpectedError => AppSW::Deny,
            ParserError::InvalidScalar => AppSW::InvalidScalar,
        }
    }
}

impl From<AppSW> for Reply {
    fn from(sw: AppSW) -> Reply {
        Reply(sw as u16)
    }
}
