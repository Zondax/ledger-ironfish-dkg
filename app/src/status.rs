#[cfg(feature = "ledger")]
use ledger_device_sdk::io::{Reply, StatusWords};

use nom::error::ErrorKind;

use crate::{ironfish::errors::IronfishError, parser::ParserError};

// Application status words.
#[repr(u16)]
#[derive(Clone, Copy, PartialEq)]
pub enum AppSW {
    Deny = 0x6985,
    WrongP1P2 = 0x6A86,
    InsNotSupported = 0x6D00,
    ClaNotSupported = 0x6E00,
    AddrDisplayFail = 0xB002,
    TxWrongLength = 0xB004,
    TxParsingFail = 0xB005,
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
    InvalidDkgStatus = 0xB022,
    InvalidDkgKeysVersion = 0xB023,
    TooManyParticipants = 0xB024,
    InvalidTxHash = 0xB025,
    InvalidToken = 0xB026,
    ErrExpertModeMustBeEnabled = 0xB027,
    #[cfg(feature = "ledger")]
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
            IronfishError::ErrExpertModeMustBeEnabled => AppSW::ErrExpertModeMustBeEnabled,
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
            ParserError::BufferFull => AppSW::BufferOutOfBounds,
            ParserError::InvalidTokenList => AppSW::InvalidPublicPackage,
            ParserError::UnknownToken => AppSW::InvalidToken,
            ParserError::ErrExpertModeMustBeEnabled => AppSW::ErrExpertModeMustBeEnabled,
        }
    }
}

#[cfg(feature = "ledger")]
impl From<AppSW> for Reply {
    fn from(sw: AppSW) -> Reply {
        Reply(sw as u16)
    }
}

impl From<ErrorKind> for AppSW {
    fn from(err: ErrorKind) -> Self {
        match err {
            ErrorKind::Eof => AppSW::BufferOutOfBounds,
            ErrorKind::TooLarge => AppSW::BufferOutOfBounds,
            ErrorKind::Tag => AppSW::TxParsingFail,
            _ => AppSW::Deny,
        }
    }
}

impl From<ParserError> for IronfishError {
    fn from(error: ParserError) -> Self {
        match error {
            ParserError::Ok => IronfishError::InvalidData, // Ok shouldn't really be converted to an error, but we need to handle it
            ParserError::UnexpectedBufferEnd => IronfishError::InvalidData,
            ParserError::ValueOutOfRange => IronfishError::IllegalValue,
            ParserError::OperationOverflows => IronfishError::IllegalValue,
            ParserError::UnexpectedValue => IronfishError::InvalidData,
            ParserError::UnexpectedType => IronfishError::InvalidData,
            ParserError::InvalidTxVersion => IronfishError::InvalidTransactionVersion,
            ParserError::InvalidKey => IronfishError::InvalidSigningKey,
            ParserError::InvalidAffinePoint => IronfishError::InvalidDiversificationPoint,
            ParserError::InvalidScalar => IronfishError::InvalidScalar,
            ParserError::InvalidTypeId => IronfishError::InvalidData,
            ParserError::InvalidSpend => IronfishError::InvalidSpendProof,
            ParserError::InvalidOuptut => IronfishError::InvalidOutputProof,
            ParserError::InvalidMint => IronfishError::InvalidMintProof,
            ParserError::InvalidBurn => IronfishError::InvalidData,
            ParserError::BufferFull => IronfishError::InvalidData,
            ParserError::InvalidTokenList => IronfishError::InvalidAssetIdentifier,
            ParserError::UnexpectedError => IronfishError::InvalidData,
            ParserError::UnknownToken => IronfishError::InvalidData,
            ParserError::ErrExpertModeMustBeEnabled => IronfishError::ErrExpertModeMustBeEnabled,
        }
    }
}

impl<I> nom::error::ParseError<I> for AppSW {
    fn from_error_kind(_input: I, kind: ErrorKind) -> Self {
        Self::from(kind)
    }

    // We don't have enough memory resources to use here an array with the last
    // N errors to be used as a backtrace, so that, we just propagate here the latest
    // reported error
    fn append(_input: I, _kind: ErrorKind, other: Self) -> Self {
        other
    }
}
impl From<AppSW> for nom::Err<AppSW> {
    fn from(error: AppSW) -> Self {
        nom::Err::Error(error)
    }
}

impl From<nom::Err<Self>> for AppSW {
    fn from(e: nom::Err<Self>) -> Self {
        match e {
            nom::Err::Error(e) => e,
            nom::Err::Failure(e) => e,
            nom::Err::Incomplete(_) => Self::BufferOutOfBounds,
        }
    }
}
