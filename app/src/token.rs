use serde::Deserialize;

use crate::parser::ParserError;
use crate::token_info::TOKEN_LIST;
#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct TokenList<'a> {
    #[serde(rename = "schemaVersion")]
    _schema_version: u32,
    #[serde(bound(deserialize = "'de: 'a"))]
    pub assets: [TokenInfo<'a>; 1],
}

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct TokenInfo<'a> {
    #[serde(borrow)]
    pub identifier: &'a str,
    #[serde(borrow)]
    pub symbol: &'a str,
    pub decimals: u8,
}

pub fn get_token_list() -> Result<TokenList<'static>, ParserError> {
    serde_json_core::from_str::<TokenList>(TOKEN_LIST)
        .map(|(t, _)| t)
        .map_err(|_| ParserError::InvalidTokenList)
}

impl<'a> TokenList<'a> {
    pub fn token(&self, asset_id: &str) -> Option<&TokenInfo> {
        self.assets
            .iter()
            .find(|asset| asset.identifier == asset_id)
    }

    pub fn toke_by_symbol(&self, symbol: &str) -> Option<&TokenInfo> {
        self.assets.iter().find(|asset| asset.symbol == symbol)
    }
}

#[cfg(test)]
mod token_list_test {
    use super::*;

    #[test]
    fn parse_token_list() {
        let token_list = get_token_list().unwrap();
        assert_eq!(token_list.assets.len(), 1);
        assert_eq!(token_list._schema_version, 2);
    }
}
