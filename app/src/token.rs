use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::parser::ParserError;
use crate::token_info::TOKEN_LIST;
#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct TokenList<'a> {
    schema_version: u32,
    #[serde(bound(deserialize = "'de: 'a"))]
    assets: [TokenInfo<'a>; 3],
}

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct TokenInfo<'a> {
    #[serde(borrow)]
    pub identifier: &'a str,
    #[serde(borrow)]
    pub symbol: &'a str,
    pub decimals: u8,
    #[serde(skip)]
    logo_uri: &'a str,
    #[serde(skip)]
    website: &'a str,
}

pub fn get_token_list() -> Result<TokenList<'static>, ParserError> {
    let t = serde_json_core::from_str::<TokenList>(TOKEN_LIST)
        .map(|(t, _)| t)
        .map_err(|_| ParserError::InvalidTokenList)?;
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
}

#[cfg(test)]
mod token_list_test {
    use super::*;

    #[test]
    fn parse_token_list() {
        let token_list = get_token_list().unwrap();
        assert_eq!(token_list.assets.len(), 3);
        assert_eq!(token_list.schema_version, 2);
    }
}
