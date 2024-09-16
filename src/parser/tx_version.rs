use super::ParserError;

#[derive(Copy, PartialEq, Clone)]
pub enum TransactionVersion {
    V1,
    V2,
}

impl TransactionVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionVersion::V1 => "V1",
            TransactionVersion::V2 => "V2",
        }
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
