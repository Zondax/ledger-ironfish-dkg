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

#![no_std]
#![no_main]

mod parser;
mod utils;
mod app_ui {
    pub mod menu;
    pub mod generic;
}
mod ironfish {
    pub mod constants;
    pub mod errors;
    pub mod multisig;
    pub mod public_address;
    pub mod sapling;
    pub mod view_keys;
}

mod instructions;
pub use instructions::Instruction;
mod status;
pub use status::AppSW;

pub use parser::{
    Burn, FromBytes, Mint, ObjectList, Output, Spend, Transaction, TransactionVersion,
};

mod bolos;
mod handlers;
pub(crate) use bolos::{app_canary, zlog, zlog_stack};

mod nvm {
    pub mod buffer;
    pub mod dkg_keys;
}

pub mod accumulator;
mod context;

pub mod crypto;
use crypto::{ConstantKey, Epk};

use crate::handlers::handle_apdu;
use app_ui::menu::ui_menu_main;

use ledger_device_sdk::io::{ApduHeader, Comm, Event, Reply, StatusWords};
#[cfg(feature = "pending_review_screen")]
#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_device_sdk::ui::gadgets::display_pending_review;

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

// Required for using String, Vec, format!...
extern crate alloc;

use crate::context::TxContext;
#[cfg(any(target_os = "stax", target_os = "flex"))]
use ledger_device_sdk::nbgl::{init_comm, NbglReviewStatus, StatusType};

const APP_CLA: u8 = 0x63;

#[no_mangle]
extern "C" fn sample_main() {
    // Create the communication manager, and configure it to accept only APDU from the 0xe0 class.
    // If any APDU with a wrong class value is received, comm will respond automatically with
    // BadCla status word.
    let mut comm = Comm::new().set_expected_cla(APP_CLA);

    // Initialize reference to Comm instance for NBGL
    // API calls.
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    init_comm(&mut comm);

    // Developer mode / pending review popup
    // must be cleared with user interaction
    #[cfg(feature = "pending_review_screen")]
    #[cfg(not(any(target_os = "stax", target_os = "flex")))]
    display_pending_review(&mut comm);

    let mut tx_ctx = TxContext::new();

    loop {
        // Wait for either a specific button push to exit the app
        // or an APDU command
        if let Event::Command(ins) = ui_menu_main(&mut comm) {
            let result = handle_apdu(&mut comm, &ins, &mut tx_ctx);
            let _status: AppSW = match result {
                Ok(()) => {
                    comm.reply_ok();
                    AppSW::Ok
                }
                Err(sw) => {
                    tx_ctx.reset();
                    comm.reply(sw);
                    sw
                }
            };
        }
    }
}
