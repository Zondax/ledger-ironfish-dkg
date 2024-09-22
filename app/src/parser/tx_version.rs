use super::ParserError;

#[repr(u8)]
#[derive(Copy, PartialEq, Clone, PartialOrd)]
#[cfg_attr(test, derive(Debug))]
pub enum TransactionVersion {
    V1 = 1,
    V2 = 2,
}

impl TransactionVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionVersion::V1 => "V1",
            TransactionVersion::V2 => "V2",
        }
    }

    pub fn has_mint_transfer_ownership_to(self) -> bool {
        self >= Self::V2
    }
}

impl TryFrom<u8> for TransactionVersion {
    type Error = ParserError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(TransactionVersion::V1),
            2 => Ok(TransactionVersion::V2),
            _ => Err(ParserError::InvalidTxVersion),
        }
    }
}
