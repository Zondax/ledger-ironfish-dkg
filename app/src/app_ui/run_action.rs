/*****************************************************************************
 *   Ledger App Boilerplate Rust.
 *   (c) 2023 Ledger SAS.
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 *****************************************************************************/

use crate::{ironfish::view_keys::OutgoingViewKey, ledger::zlog_stack, AppSW, Transaction};

use alloc::vec::Vec;
#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_device_sdk::ui::{
    bitmaps::{CROSSMARK, EYE, VALIDATE_14},
    gadgets::{Field, MultiFieldReview},
};

#[cfg(any(target_os = "stax", target_os = "flex"))]
use ledger_device_sdk::nbgl::{Field, NbglGlyph, NbglReview, TransactionType};

use crate::bolos::app_canary;
use crate::utils::int_to_str;
#[cfg(any(target_os = "stax", target_os = "flex"))]
use include_gif::include_gif;

#[inline(never)]
pub fn ui_review_transaction<'a>(
    transaction: &'a Transaction<'a>,
    ovk: &OutgoingViewKey,
) -> Result<bool, AppSW> {
    zlog_stack("ui_review_transaction***\0");

    #[cfg(not(any(target_os = "stax", target_os = "flex")))]
    {
        let field_pairs = transaction.review_fields(ovk)?;

        // Create a vector to hold the Field structs
        let fields: Vec<Field> = field_pairs
            .iter()
            .map(|(name, value)| Field {
                name: name.as_str(),
                value: value.as_str(),
            })
            .collect();

        crate::bolos::zlog_num("num_fields_ui \0", fields.len() as u32);

        let review_message = ["Review", "Transaction"];
        let my_review = MultiFieldReview::new(
            fields.as_slice(),
            &review_message,
            Some(&EYE),
            "Approve",
            Some(&VALIDATE_14),
            "Reject",
            Some(&CROSSMARK),
        );

        Ok(my_review.show())
    }

    #[cfg(any(target_os = "stax", target_os = "flex"))]
    {
        #[cfg(target_os = "stax")]
        const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("stax_icon.gif", NBGL));
        #[cfg(target_os = "flex")]
        const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("flex_icon.gif", NBGL));

        let field_pairs = transaction.review_fields(ovk)?;

        // Create a vector to hold the Field structs
        let fields: Vec<Field> = field_pairs
            .iter()
            .map(|(name, value)| Field { name, value })
            .collect();

        let review = NbglReview::new()
            .tx_type(TransactionType::Transaction)
            .titles("Review", "Transaction", "Approve Transaction?")
            .glyph(&FERRIS);

        Ok(review.show(&fields))
    }
}

#[inline(never)]
pub fn ui_review_get_identity(i_index: u8) -> Result<bool, AppSW> {
    zlog_stack("s review_get_identity\0");
    app_canary();

    let i_index_str = int_to_str(i_index);
    let fields: [Field; 1] = [Field {
        name: "Identity Num.",
        value: i_index_str.as_str(),
    }];

    app_canary();
    ui_review("Get Identity", "", "Accept operation?", &fields, true)
}

#[inline(never)]
pub fn ui_review_get_keys(data: &Vec<u8>, key_type: u8) -> Result<bool, AppSW> {
    zlog_stack("s ui_review_get_keys\0");
    app_canary();

    let title = "Get Keys";
    let finish_title = "Accept operation?";

    match key_type {
        0 => {
            let mut public_address_hex_str = hex::encode(data);
            public_address_hex_str.insert_str(0, "0x");
            let fields: [Field; 1] = [Field {
                name: "Public Address",
                value: public_address_hex_str.as_str(),
            }];
            ui_review(title, "", finish_title, &fields, true)
        }
        1 => {
            let data_slice = data.as_slice();
            let mut view_key_hex_str = hex::encode(data_slice[0..64].as_ref());
            view_key_hex_str.insert_str(0, "0x");
            let mut ivk_hex_str = hex::encode(data_slice[64..96].as_ref());
            ivk_hex_str.insert_str(0, "0x");
            let mut ovk_hex_str = hex::encode(data_slice[96..128].as_ref());
            ovk_hex_str.insert_str(0, "0x");

            let fields: [Field; 3] = [
                Field {
                    name: "ViewKey",
                    value: view_key_hex_str.as_str(),
                },
                Field {
                    name: "IVK",
                    value: ivk_hex_str.as_str(),
                },
                Field {
                    name: "OVK",
                    value: ovk_hex_str.as_str(),
                },
            ];
            ui_review(title, "", finish_title, &fields, true)
        }
        2 => {
            let data_slice = data.as_slice();
            let mut auth_key_hex_str = hex::encode(data_slice[0..32].as_ref());
            auth_key_hex_str.insert_str(0, "0x");
            let mut proof_auth_key_hex_str = hex::encode(data_slice[32..64].as_ref());
            proof_auth_key_hex_str.insert_str(0, "0x");

            let fields: [Field; 2] = [
                Field {
                    name: "AuthKey",
                    value: auth_key_hex_str.as_str(),
                },
                Field {
                    name: "ProofAuthKey",
                    value: proof_auth_key_hex_str.as_str(),
                },
            ];
            ui_review(title, "", finish_title, &fields, true)
        }
        _ => Err(AppSW::InvalidKeyType),
    }
}

#[inline(never)]
pub fn ui_review_get_current_identity(i_index: u8) -> Result<bool, AppSW> {
    zlog_stack("s review_current_identity\0");
    app_canary();

    let i_index_str = int_to_str(i_index);
    let fields: [Field; 1] = [Field {
        name: "Identity Num.",
        value: i_index_str.as_str(),
    }];

    app_canary();
    ui_review(
        "Get Current",
        "Identity",
        "Accept operation?",
        &fields,
        true,
    )
}

#[inline(never)]
pub fn ui_review_dkg_round1(i_index: u8, min_signers: u8, participants: u8) -> Result<bool, AppSW> {
    zlog_stack("s review_dkg_round1\0");

    let i_index_str = int_to_str(i_index);
    let min_signers_str = int_to_str(min_signers);
    let participants_str = int_to_str(participants);

    let fields: [Field; 3] = [
        Field {
            name: "Identity Num.",
            value: i_index_str.as_str(),
        },
        Field {
            name: "Participants",
            value: participants_str.as_str(),
        },
        Field {
            name: "Min. Signers",
            value: min_signers_str.as_str(),
        },
    ];

    ui_review("Round 1", "", "Accept operation?", &fields, true)
}

#[inline(never)]
pub fn ui_review_dkg_round2(i_index: u8, round1_public_package_len: u8) -> Result<bool, AppSW> {
    zlog_stack("s review_dkg_round2\0");
    app_canary();

    let i_index_str = int_to_str(i_index);
    let round1_public_package_len_str = int_to_str(round1_public_package_len);

    let fields: [Field; 2] = [
        Field {
            name: "Identity Num.",
            value: i_index_str.as_str(),
        },
        Field {
            name: "Packages from R1",
            value: round1_public_package_len_str.as_str(),
        },
    ];

    ui_review("Round 2", "", "Accept operation?", &fields, true)
}

#[inline(never)]
pub fn ui_review_backup_keys(
    public_address: Vec<u8>,
    participants: u8,
    min_signers: u8,
) -> Result<bool, AppSW> {
    zlog_stack("s review_backup_keys\0");
    app_canary();

    let participants_str = int_to_str(participants);
    let min_signers_str = int_to_str(min_signers);
    let mut public_address_hex_str = hex::encode(public_address);
    public_address_hex_str.insert_str(0, "0x");

    let fields: [Field; 3] = [
        Field {
            name: "Public Address",
            value: public_address_hex_str.as_str(),
        },
        Field {
            name: "Participants",
            value: participants_str.as_str(),
        },
        Field {
            name: "Min. Signers",
            value: min_signers_str.as_str(),
        },
    ];

    ui_review("Backup Keys", "", "Accept operation?", &fields, true)
}

#[inline(never)]
pub fn ui_review_dkg_round3(
    i_index: u8,
    round1_public_package_len: u8,
    round2_public_package_len: u8,
    participants_len: u8,
    gsk_len: u8,
) -> Result<bool, AppSW> {
    zlog_stack("s review_dkg_round3\0");
    app_canary();

    let i_index_str = int_to_str(i_index);
    let round1_public_package_len_str = int_to_str(round1_public_package_len);
    let round2_public_package_len_str = int_to_str(round2_public_package_len);
    let participants_len_str = int_to_str(participants_len);
    let gsk_len_str = int_to_str(gsk_len);

    let fields: [Field; 5] = [
        Field {
            name: "Identity Num.",
            value: i_index_str.as_str(),
        },
        Field {
            name: "Participants",
            value: participants_len_str.as_str(),
        },
        Field {
            name: "Packages from R1",
            value: round1_public_package_len_str.as_str(),
        },
        Field {
            name: "Packages from R2",
            value: round2_public_package_len_str.as_str(),
        },
        Field {
            name: "Group Shared Keys",
            value: gsk_len_str.as_str(),
        },
    ];

    ui_review("Round 3", "", "Accept operation?", &fields, true)
}

#[inline(never)]
pub fn ui_review_restore_keys(
    public_address: Vec<u8>,
    participants: u8,
    min_signers: u8,
) -> Result<bool, AppSW> {
    zlog_stack("s review_restore_keys\0");
    app_canary();

    let participants_str = int_to_str(participants);
    let min_signers_str = int_to_str(min_signers);
    let mut public_address_hex_str = hex::encode(public_address);
    public_address_hex_str.insert_str(0, "0x");

    let fields: [Field; 3] = [
        Field {
            name: "Public Address",
            value: public_address_hex_str.as_str(),
        },
        Field {
            name: "Participants",
            value: participants_str.as_str(),
        },
        Field {
            name: "Min. Signers",
            value: min_signers_str.as_str(),
        },
    ];

    ui_review("Restore Keys", "", "Accept operation?", &fields, true)
}

#[inline(never)]
pub fn ui_review<'a>(
    title: &'a str,
    _subtitle: &'a str,
    _finish_title: &'a str,
    fields: &'a [Field<'a>],
    _light: bool,
) -> Result<bool, AppSW> {
    zlog_stack("x ui_review\0");
    app_canary();

    #[cfg(not(any(target_os = "stax", target_os = "flex")))]
    {
        let review_messages = [title];
        let review = MultiFieldReview::new(
            fields,
            &review_messages,
            Some(&EYE),
            "Approve",
            Some(&VALIDATE_14),
            "Reject",
            Some(&CROSSMARK),
        );

        Ok(review.show())
    }

    #[cfg(any(target_os = "stax", target_os = "flex"))]
    {
        #[cfg(target_os = "stax")]
        const ICON: NbglGlyph = NbglGlyph::from_include(include_gif!("stax_icon.gif", NBGL));
        #[cfg(target_os = "flex")]
        const ICON: NbglGlyph = NbglGlyph::from_include(include_gif!("flex_icon.gif", NBGL));

        let review = NbglReview::new()
            .tx_type(TransactionType::Operation)
            .light()
            .titles(title, _subtitle, _finish_title)
            .glyph(&ICON);

        Ok(review.show(fields))
    }
}
