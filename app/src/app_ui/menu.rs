/*****************************************************************************
 *   Ledger App Ironfish Rust.
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

use include_gif::include_gif;
use ledger_device_sdk::io::{Comm, Event};

#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_device_sdk::ui::{
    bitmaps::{Glyph, BACK, CERTIFICATE, DASHBOARD},
    gadgets::{EventOrPageIndex, MultiPageMenu, Page},
};

#[cfg(any(target_os = "stax", target_os = "flex"))]
use ledger_device_sdk::nbgl::{NbglGlyph, NbglHomeAndSettings};

use crate::Instruction;

#[cfg(not(any(target_os = "stax", target_os = "flex")))]
pub fn ui_menu_main(comm: &mut Comm) -> Event<Instruction> {
    const APP_ICON: Glyph = Glyph::from_include(include_gif!("nanox_icon.gif"));

    let mut first_page_label: [&str; 2];

    if env!("PRODUCTION_BUILD") == "0" {
        first_page_label = ["Ironfish DKG DEMO", "DO NOT USE"];
    } else {
        first_page_label = ["Ironfish DKG", "Ready"];
    }

    let pages = [
        &Page::from((first_page_label, &APP_ICON)),
        &Page::from((["Ironfish DKG", env!("APPVERSION_STR")], true, true)),
        &Page::from((["Developed by", "Zondax.ch"], true, true)),
        &Page::from((["License", "Apache 2.0"], true, true)),
        &Page::from(("Quit", &DASHBOARD)),
    ];

    loop {
        match MultiPageMenu::new(comm, &pages).show() {
            EventOrPageIndex::Event(e) => return e,
            EventOrPageIndex::Index(4) => ledger_device_sdk::exit_app(0),
            EventOrPageIndex::Index(_) => (),
        }
    }
}

#[cfg(any(target_os = "stax", target_os = "flex"))]
pub fn ui_menu_main(_: &mut Comm) -> Event<Instruction> {
    // Load glyph from 64x64 4bpp gif file with include_gif macro. Creates an NBGL compatible glyph.

    #[cfg(target_os = "stax")]
    const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("stax_icon.gif", NBGL));
    #[cfg(target_os = "flex")]
    const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("flex_icon.gif", NBGL));

    let name: &str;
    if env!("PRODUCTION_BUILD") == "0" {
        name = "Ironfish DKG DEMO";
    } else {
        name = "Ironfish DKG";
    }

    // Display the home screen.
    NbglHomeAndSettings::new()
        .glyph(&FERRIS)
        .infos(name, env!("APPVERSION_STR"), "Zondax AG")
        .show()
}
