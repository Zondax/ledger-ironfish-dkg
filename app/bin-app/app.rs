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
#![cfg(feature = "ledger")]
#![no_std]
#![no_main]

use ironfish_dkg::{context::TxContext, AppSW};
use ledger_device_sdk::io::Comm;

#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_device_sdk::io::Event;

use ironfish_dkg::ledger::*;

#[cfg(any(target_os = "stax", target_os = "flex"))]
use ledger_device_sdk::nbgl::{init_comm, NbglReviewStatus, StatusType};

#[cfg(any(target_os = "stax", target_os = "flex"))]
use ironfish_dkg::Instruction;

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

const APP_CLA: u8 = 0x63;

#[no_mangle]
extern "C" fn sample_main() {
    // Create the communication manager, and configure it to accept only APDU from the 0x63 class.
    // If any APDU with a wrong class value is received, comm will respond automatically with
    // BadCla status word.
    let mut comm = Comm::new().set_expected_cla(APP_CLA);

    let mut tx_ctx = TxContext::new();

    #[cfg(any(target_os = "stax", target_os = "flex"))]
    {
        // Initialize reference to Comm instance for NBGL
        // API calls.
        init_comm(&mut comm);
        tx_ctx.home = ui_menu_main(&mut comm);
        tx_ctx.home.show_and_return();
    }

    loop {
        #[cfg(any(target_os = "stax", target_os = "flex"))]
        let ins: Instruction = comm.next_command();

        #[cfg(not(any(target_os = "stax", target_os = "flex")))]
        let ins = if let Event::Command(ins) = ui_menu_main(&mut comm) {
            ins
        } else {
            continue;
        };

        let _status = match handle_apdu(&mut comm, &ins, &mut tx_ctx) {
            Ok(()) => AppSW::Ok,
            Err(sw) => sw,
        };

        #[cfg(any(target_os = "stax", target_os = "flex"))]
        show_status_and_home_if_needed(&ins, &mut tx_ctx, &_status);

        if _status == AppSW::Ok {
            comm.reply_ok();
        } else {
            // On any error we return, we reset the buffer to receive mode
            tx_ctx.reset_to_receive();
            comm.reply(_status);
        }
    }
}

// Based on the instruction received, whether it is still accumulating data in the input buffer or not,
// and the status code answered (deny/approve only, no processing errors or so), we should decide if
// a status message is displayed in the screen, and whether we need to display the menu again
#[cfg(any(target_os = "stax", target_os = "flex"))]
fn show_status_and_home_if_needed(ins: &Instruction, tx_ctx: &mut TxContext, status: &AppSW) {
    let (return_home, show_status, status_type) = match (ins, status) {
        (Instruction::DkgBackupKeys, AppSW::Deny | AppSW::Ok) => {
            (true, false, StatusType::Operation)
        }
        (Instruction::DkgRestoreKeys { .. }, AppSW::Deny | AppSW::Ok) if tx_ctx.done => {
            (true, false, StatusType::Operation)
        }
        (Instruction::DkgGetIdentity { review: true }, AppSW::Deny | AppSW::Ok) => {
            (true, false, StatusType::Operation)
        }
        (Instruction::DkgRound1 { .. }, AppSW::Deny | AppSW::Ok) if tx_ctx.done => {
            (true, true, StatusType::Operation)
        }
        (Instruction::DkgRound2 { .. }, AppSW::Deny | AppSW::Ok) if tx_ctx.done => {
            (true, true, StatusType::Operation)
        }
        (Instruction::DkgRound3Min { .. }, AppSW::Deny | AppSW::Ok) if tx_ctx.done => {
            (true, true, StatusType::Operation)
        }
        (Instruction::DkgGetKeys { review: true, .. }, AppSW::Deny | AppSW::Ok) => {
            (true, false, StatusType::Address)
        }
        (Instruction::ReviewTx { .. }, AppSW::Deny | AppSW::Ok) if tx_ctx.done => {
            (true, true, StatusType::Transaction)
        }
        (_, _) => (false, false, StatusType::Operation),
    };

    if show_status {
        let success = *status == AppSW::Ok;
        NbglReviewStatus::new()
            .status_type(status_type)
            .show(success);
        tx_ctx.home.show_and_return();
    } else if return_home {
        tx_ctx.home.show_and_return();
    }
}
