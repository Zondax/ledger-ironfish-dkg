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
use alloc::string::String;
use include_gif::include_gif;
use ledger_device_sdk::io::Comm;
#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_device_sdk::io::Event;

#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_device_sdk::ui::{
    bitmaps::{Glyph, DASHBOARD},
    gadgets::{EventOrPageIndex, MultiPageMenu, Page},
};

#[cfg(any(target_os = "stax", target_os = "flex"))]
use ledger_device_sdk::nbgl::{NbglGlyph, NbglHomeAndSettings};

use crate::nvm::settings::Settings;
#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use crate::Instruction;

#[cfg(not(any(target_os = "stax", target_os = "flex")))]
#[inline(never)]
pub fn ui_menu_main(comm: &mut Comm) -> Event<Instruction> {
    const APP_ICON: Glyph = Glyph::from_include(include_gif!("nanox_icon.gif"));

    let mut _first_page_label: [&str; 2];

    let production_build = option_env!("PRODUCTION_BUILD").unwrap_or("1");

    let app_version_value = option_env!("APPVERSION").unwrap_or("0.0.0");
    let mut app_version = String::from("v");
    app_version.push_str(app_version_value);

    if production_build == "0" {
        _first_page_label = ["Ironfish DKG DEMO", "DO NOT USE"];
    } else {
        _first_page_label = ["Ironfish DKG", "Ready"];
    }

    let mut last_page = 0;
    loop {
        let expert_mode_label = match Settings.app_expert_mode() {
            true => "Enabled",
            false => "Disabled",
        };

        let pages = [
            &Page::from((_first_page_label, &APP_ICON)),
            &Page::from((["Expert Mode", expert_mode_label], true, true)),
            &Page::from((["Ironfish DKG", app_version.as_str()], true, true)),
            &Page::from((["Developed by", "Zondax.ch"], true, true)),
            &Page::from((["License", "Apache 2.0"], true, true)),
            &Page::from(("Quit", &DASHBOARD)),
        ];

        match MultiPageMenu::new(comm, &pages).show_from(last_page) {
            EventOrPageIndex::Event(e) => return e,
            EventOrPageIndex::Index(page_index) => {
                match page_index {
                    1 => Settings.toggle_expert_mode(),
                    5 => ledger_device_sdk::exit_app(0),
                    _ => (),
                }

                last_page = page_index
            }
        }
    }
}

#[cfg(any(target_os = "stax", target_os = "flex"))]
#[inline(never)]
pub fn ui_menu_main(_: &mut Comm) -> NbglHomeAndSettings {
    #[cfg(target_os = "stax")]
    const APP_ICON: NbglGlyph = NbglGlyph::from_include(include_gif!("stax_icon.gif", NBGL));
    #[cfg(target_os = "flex")]
    const APP_ICON: NbglGlyph = NbglGlyph::from_include(include_gif!("flex_icon.gif", NBGL));

    let settings_strings = [["Expert Mode", ""]];
    let mut settings: Settings = Default::default();

    let production_build = option_env!("PRODUCTION_BUILD").unwrap_or("1");

    let app_version_value = option_env!("APPVERSION").unwrap_or("0.0.0");
    let mut app_version = String::from("v");
    app_version.push_str(app_version_value);

    let name: &str = if production_build == "0" {
        "Ironfish DKG DEMO"
    } else {
        "Ironfish DKG"
    };

    // Display the home screen.
    NbglHomeAndSettings::new()
        .glyph(&APP_ICON)
        .settings(settings.get_mut(), &settings_strings)
        .infos(name, app_version.as_str(), "Zondax AG")
}
