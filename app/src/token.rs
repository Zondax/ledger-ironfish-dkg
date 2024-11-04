use serde::Deserialize;

use crate::parser::ParserError;
use crate::token_info::TOKEN_LIST;
#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct TokenList<'a> {
    #[serde(rename = "schemaVersion")]
    _schema_version: u32,
    #[serde(bound(deserialize = "'de: 'a"))]
    assets: [TokenInfo<'a>; 1],
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
    let t = serde_json_core::from_str::<TokenList>(TOKEN_LIST)
        .map(|(t, _)| t)
        .unwrap();
    // .map_err(|_| ParserError::InvalidTokenList)?;
    Ok(t)
}

impl<'a> TokenList<'a> {
    pub fn get_ticker(&self, asset_id: &str) -> Option<&str> {
        self.assets.iter().find_map(|asset| {
            if asset.identifier == asset_id {
                Some(asset.symbol)
            } else {
                None
            }
        })
    }

    pub fn token(&self, asset_id: &str) -> Option<&TokenInfo> {
        self.assets
            .iter()
            .find(|asset| asset.identifier == asset_id)
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
