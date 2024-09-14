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

use crate::AppSW;
use alloc::format;

#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_device_sdk::ui::{
    bitmaps::{CROSSMARK, EYE, VALIDATE_14},
    gadgets::{Field, MultiFieldReview},
};

#[cfg(any(target_os = "stax", target_os = "flex"))]
use ledger_device_sdk::nbgl::{NbglAddressReview, NbglGlyph};

#[cfg(any(target_os = "stax", target_os = "flex"))]
use include_gif::include_gif;

pub fn ui_run_action<'a>(review_message: &'a [&'a str]) -> Result<bool, AppSW> {
    #[cfg(not(any(target_os = "stax", target_os = "flex")))]
    {
        let my_field:[Field;0] = [];

        let my_review = MultiFieldReview::new(
            &my_field,
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
        // Load glyph from 64x64 4bpp gif file with include_gif macro. Creates an NBGL compatible glyph.
        const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("crab_64x64.gif", NBGL));
        // Display the address confirmation screen.
        Ok(NbglAddressReview::new()
            .glyph(&FERRIS)
            .verify_str("Verify CRAB address")
            .show(&addr_hex))
    }
}