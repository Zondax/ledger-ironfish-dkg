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

mod utils;
mod app_ui {
    pub mod menu;
}
mod ironfish{
    pub mod sapling;
    pub mod constants;
    pub mod view_keys;
    pub mod errors;
    pub mod multisig;
    pub mod public_address;
}

mod handlers {
    pub mod get_version;
    pub mod dkg_get_identity;
    pub mod dkg_round_1;
    pub mod dkg_round_2;
    pub mod dkg_round_3;
    pub mod dkg_get_keys;
    pub mod dkg_commitments;
    pub mod dkg_nonces;
    pub mod dkg_sign;
    pub mod dkg_get_public_package;
}

mod nvm {
    pub mod buffer;
    pub mod dkg_keys;
}

mod context;
pub mod accumulator;

use app_ui::menu::ui_menu_main;
use handlers::{
    dkg_get_identity::handler_dkg_get_identity,
    dkg_round_1::handler_dkg_round_1,
    dkg_round_2::handler_dkg_round_2,
    dkg_round_3::handler_dkg_round_3,
    dkg_get_keys::handler_dkg_get_keys,
    get_version::handler_get_version,
    dkg_commitments::handler_dkg_commitments,
    dkg_nonces::handler_dkg_nonces,
    dkg_sign::handler_dkg_sign,
};

use ledger_device_sdk::io::{ApduHeader, Comm, Event, Reply, StatusWords};
#[cfg(feature = "pending_review_screen")]
#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_device_sdk::ui::gadgets::display_pending_review;

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

// Required for using String, Vec, format!...
extern crate alloc;

#[cfg(any(target_os = "stax", target_os = "flex"))]
use ledger_device_sdk::nbgl::{init_comm, NbglReviewStatus, StatusType};
use crate::context::TxContext;
use crate::handlers::dkg_get_public_package::handler_dkg_get_public_package;

// Application status words.
#[repr(u16)]
#[derive(Clone, Copy, PartialEq)]
pub enum AppSW {
    Deny = 0x6985,
    WrongP1P2 = 0x6A86,
    InsNotSupported = 0x6D00,
    ClaNotSupported = 0x6E00,
    TxDisplayFail = 0xB001,
    AddrDisplayFail = 0xB002,
    TxWrongLength = 0xB004,
    TxParsingFail = 0xB005,
    TxHashFail = 0xB006,
    TxSignFail = 0xB008,
    KeyDeriveFail = 0xB009,
    VersionParsingFail = 0xB00A,
    DkgRound2Fail = 0xB00B,
    DkgRound3Fail = 0xB00C,
    InvalidKeyType = 0xB00D,
    InvalidIdentity = 0xB00E,
    InvalidPayload = 0xB00F,
    BufferOutOfBounds = 0xB010,
    InvalidSigningPackage = 0xB011,
    InvalidRandomizer = 0xB012,
    InvalidSigningNonces = 0xB013,
    InvalidIdentityIndex = 0xB014,
    InvalidKeyPackage = 0xB015,
    InvalidPublicPackage = 0xB016,
    InvalidGroupSecretKey = 0xB017,
    WrongApduLength = StatusWords::BadLen as u16,
    Ok = 0x9000,
}

impl From<AppSW> for Reply {
    fn from(sw: AppSW) -> Reply {
        Reply(sw as u16)
    }
}

/// Possible input commands received through APDUs.
pub enum Instruction {
    GetVersion,
    GetAppName,
    DkgGetIdentity,
    DkgGetPublicPackage,
    DkgRound1 { chunk: u8 },
    DkgRound2 { chunk: u8 },
    DkgRound3 { chunk: u8 },
    DkgCommitments { chunk: u8 },
    DkgSign { chunk: u8 },
    DkgGetKeys { key_type: u8 },
    DkgNonces { chunk: u8 },
}

impl TryFrom<ApduHeader> for Instruction {
    type Error = AppSW;

    /// APDU parsing logic.
    ///
    /// Parses INS, P1 and P2 bytes to build an [`Instruction`]. P1 and P2 are translated to
    /// strongly typed variables depending on the APDU instruction code. Invalid INS, P1 or P2
    /// values result in errors with a status word, which are automatically sent to the host by the
    /// SDK.
    ///
    /// This design allows a clear separation of the APDU parsing logic and commands handling.
    ///
    /// Note that CLA is not checked here. Instead the method [`Comm::set_expected_cla`] is used in
    /// [`sample_main`] to have this verification automatically performed by the SDK.
    fn try_from(value: ApduHeader) -> Result<Self, Self::Error> {
        match (value.ins, value.p1, value.p2) {
            (3, 0, 0) => Ok(Instruction::GetVersion),
            (4, 0, 0) => Ok(Instruction::GetAppName),
            (16, 0, 0) => Ok(Instruction::DkgGetIdentity),
            (17, 0..=2, 0) => {
                Ok(Instruction::DkgRound1 {
                    chunk: value.p1
                })
            },
            (18, 0..=2, 0) => {
                Ok(Instruction::DkgRound2 {
                    chunk: value.p1
                })
            },
            (19, 0..=2, 0) => {
                Ok(Instruction::DkgRound3 {
                    chunk: value.p1
                })
            },
            (20, 0..=2, 0) => {
                Ok(Instruction::DkgCommitments {
                    chunk: value.p1
                })
            },
            (21, 0..=2, 0) => {
                Ok(Instruction::DkgSign {
                    chunk: value.p1
                })
            },
            (22, 0, 0..=2) => Ok(Instruction::DkgGetKeys{
                key_type: value.p2
            }),
            (23, 0..=2, 0) => {
                Ok(Instruction::DkgNonces {
                    chunk: value.p1
                })
            },
            (24, 0..=2, 0) => {
                Ok(Instruction::DkgGetPublicPackage)
            },
            (3..=4, _, _) => Err(AppSW::WrongP1P2),
            (17..=22, _, _) => Err(AppSW::WrongP1P2),
            (_, _, _) => Err(AppSW::InsNotSupported),
        }
    }
}

#[no_mangle]
extern "C" fn sample_main() {
    // Create the communication manager, and configure it to accept only APDU from the 0xe0 class.
    // If any APDU with a wrong class value is received, comm will respond automatically with
    // BadCla status word.
    let mut comm = Comm::new().set_expected_cla(0x59);

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

            #[cfg(any(target_os = "stax", target_os = "flex"))]
            show_status_if_needed(&ins, &tx_ctx, &_status);
        }
    }
}

fn handle_apdu(comm: &mut Comm, ins: &Instruction, ctx: &mut TxContext) -> Result<(), AppSW> {
    match ins {
        Instruction::GetAppName => {
            comm.append(env!("CARGO_PKG_NAME").as_bytes());
            Ok(())
        }
        Instruction::GetVersion => handler_get_version(comm),
        Instruction::DkgGetIdentity => handler_dkg_get_identity(comm),
        Instruction::DkgRound1 { chunk } => handler_dkg_round_1(comm, *chunk, ctx),
        Instruction::DkgRound2 { chunk } => handler_dkg_round_2(comm, *chunk, ctx),
        Instruction::DkgRound3 { chunk } => handler_dkg_round_3(comm, *chunk, ctx),
        Instruction::DkgCommitments { chunk } => handler_dkg_commitments(comm, *chunk, ctx),
        Instruction::DkgSign { chunk } => handler_dkg_sign(comm, *chunk, ctx),
        Instruction::DkgGetKeys {key_type} => handler_dkg_get_keys(comm, key_type),
        Instruction::DkgNonces { chunk } => handler_dkg_nonces(comm, *chunk, ctx),
        Instruction::DkgGetPublicPackage => handler_dkg_get_public_package(comm),
    }
}
