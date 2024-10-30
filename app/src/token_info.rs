use serde::{Deserialize, Serialize};

// An untrusted token information list provided by penumbra team
// https://github.com/iron-fish/verified-assets/blob/main/mainnet.json
// We were told to used this in the meantime while a trusted token metadata
// service is being developed
pub static TOKEN_LIST: &str = r#"
{
    "schemaVersion": 2,
    "assets": [
        {
            "identifier": "51f33a2f14f92735e562dc658a5639279ddca3d5079a6d1242b2a588a9cbf44c",
            "symbol": "IRON",
            "decimals": 8,
            "logoURI": "https://ironfish.network/favicon.ico",
            "website": "https://ironfish.network"
        },
        {
            "identifier": "2cabe0bddf475a478f4b3903f8fc2d2c70b52526f991bacea8267793da63f44b",
            "symbol": "TINI",
            "decimals": 8,
            "logoURI": "https://chainport-public-files-prod.s3.amazonaws.com/61bb7212-13ab-4a9a-a7c2-16a9583d2342.svg",
            "website": "https://app.chainport.io"
        },
        {
            "identifier": "5d43c6f2bbd70ae658661ab58f138b4d54276dd9102966b486bcc0d6e3ab89a3",
            "symbol": "TINE",
            "decimals": 8,
            "logoURI": "https://chainport-public-files-prod.s3.amazonaws.com/61bb7212-13ab-4a9a-a7c2-16a9583d2342.svg",
            "website": "https://app.chainport.io"
        }
    ]
}"#;
