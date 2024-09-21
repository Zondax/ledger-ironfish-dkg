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

use alloc::{format, vec::Vec};
#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_device_sdk::ui::{
    bitmaps::{CROSSMARK, EYE, VALIDATE_14},
    gadgets::{Field, MultiFieldReview},
};

#[cfg(any(target_os = "stax", target_os = "flex"))]
use ledger_device_sdk::nbgl::{Field, NbglChoice, NbglGlyph, NbglReview, TransactionType};

#[cfg(any(target_os = "stax", target_os = "flex"))]
use include_gif::include_gif;

#[inline(never)]
pub fn ui_run_action<'a>(review_message: &'a [&'a str]) -> Result<bool, AppSW> {
    #[cfg(not(any(target_os = "stax", target_os = "flex")))]
    {
        let fields: [Field; 0] = [];

        let my_review = MultiFieldReview::new(
            &fields,
            review_message,
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

        Ok(NbglChoice::new()
            .glyph(&FERRIS)
            .show(review_message[0], "", "Approve", "Reject"))
    }
}

#[inline(never)]
pub fn ui_review_transaction<'a>(
    transaction: &'a Transaction<'a>,
    ovk: &OutgoingViewKey,
) -> Result<bool, AppSW> {
    zlog_stack("ui_review_transaction***\0");

    #[cfg(not(any(target_os = "stax", target_os = "flex")))]
    {
        let field_pairs = transaction
            .review_fields(ovk)
            .map_err(|_| AppSW::BufferOutOfBounds)?;

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

        let field_pairs = transaction
            .review_fields(ovk)
            .map_err(|_| AppSW::BufferOutOfBounds)?;

        // Create a vector to hold the Field structs
        let fields: Vec<Field> = field_pairs
            .iter()
            .map(|(name, value)| Field { name, value })
            .collect();

        let mut review = NbglReview::new()
            .tx_type(TransactionType::Transaction)
            .titles("Review", "Transaction", "Approve Transaction?")
            .glyph(&FERRIS);

        Ok(review.show(&fields, false))
    }
}

#[inline(never)]
pub fn ui_review_get_identity<'a>(i_index: u8) -> Result<bool, AppSW> {
    let review_message = &["Get Identity", ""];

    let i_index_str = format!("{}", i_index);
    let fields: [Field; 1] = [Field {
        name: "Identity Num.",
        value: i_index_str.as_str(),
    }];

    ui_review(review_message, "Accept operation?", &fields, true)
}

#[inline(never)]
pub fn ui_review_dkg_round1<'a>(
    i_index: u8,
    min_signers: u8,
    participants: u8,
) -> Result<bool, AppSW> {
    let review_message = &["Round 1", ""];

    let i_index_str = format!("{}", i_index);
    let min_signers_str = format!("{}", min_signers);
    let participants_str = format!("{}", participants);
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

    ui_review(review_message, "Accept operation?", &fields, true)
}

#[inline(never)]
pub fn ui_review_dkg_round2<'a>(i_index: u8, round1_public_package_len: u8) -> Result<bool, AppSW> {
    let review_message = &["Round 2", ""];

    let i_index_str = format!("{}", i_index);
    let round1_public_package_len_str = format!("{}", round1_public_package_len);
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

    ui_review(review_message, "Accept operation?", &fields, true)
}

#[inline(never)]
pub fn ui_review_backup_keys<'a>(
    public_address: Vec<u8>,
    participants: usize,
    min_signers: usize,
) -> Result<bool, AppSW> {
    let review_message = &["Backup Keys", ""];

    let public_address_hex_str = format!("0x{}", hex::encode(public_address));
    let min_signers_str = format!("{}", min_signers);
    let participants_str = format!("{}", participants);
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

    ui_review(review_message, "Accept operation?", &fields, true)
}

#[inline(never)]
pub fn ui_review_dkg_round3<'a>(
    i_index: u8,
    round1_public_package_len: u8,
    round2_public_package_len: u8,
    participants_len: u8,
    gsk_len: u8,
) -> Result<bool, AppSW> {
    let review_message = &["Round 3", ""];

    let i_index_str = format!("{}", i_index);
    let round1_public_package_len_str = format!("{}", round1_public_package_len);
    let round2_public_package_len_str = format!("{}", round2_public_package_len);
    let participants_len_str = format!("{}", participants_len);
    let gsk_len_str = format!("{}", gsk_len);
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

    ui_review(review_message, "Accept operation?", &fields, true)
}

#[inline(never)]
pub fn ui_review_restore_keys<'a>() -> Result<bool, AppSW> {
    let review_message = &["Restore Keys", ""];

    ui_run_action(review_message)
}

#[inline(never)]
pub fn ui_review<'a>(
    review_message: &'a [&'a str],
    finish_title: &'a str,
    fields: &'a [Field<'a>],
    light: bool,
) -> Result<bool, AppSW> {
    #[cfg(not(any(target_os = "stax", target_os = "flex")))]
    {
        let my_review = MultiFieldReview::new(
            fields,
            review_message,
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

        let mut review = NbglReview::new()
            .tx_type(TransactionType::Operation)
            .titles(review_message[0], review_message[1], finish_title)
            .glyph(&FERRIS);

        Ok(review.show(&fields, true))
    }
}
