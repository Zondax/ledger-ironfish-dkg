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

#![cfg_attr(not(test), no_std)]

extern crate alloc;
pub mod bolos;
#[cfg(feature = "ledger")]
pub mod context;
pub mod crypto;
pub mod instructions;
pub mod ironfish;
pub mod parser;
pub mod status;
#[cfg(test)]
mod test_ui;
pub mod token;
pub mod token_info;
pub mod utils;

// Public re-exports
pub use crypto::{ConstantKey, Epk};
pub use instructions::Instruction;
pub use parser::{
    Burn, FromBytes, Mint, ObjectList, Output, Spend, Transaction, TransactionVersion,
};
pub use status::AppSW;

#[cfg(feature = "ledger")]
pub mod app_ui;

#[cfg(feature = "ledger")]
pub mod handlers;

#[cfg(feature = "ledger")]
pub mod nvm;

#[cfg(feature = "ledger")]
pub mod accumulator;

pub(crate) mod rand;

#[cfg(feature = "ledger")]
pub mod ledger {
    pub use super::accumulator;
    pub use super::app_ui;
    pub use super::app_ui::menu::ui_menu_main;
    pub use super::bolos::{app_canary, zlog, zlog_stack};
    pub use super::handlers;
    pub use super::handlers::handle_apdu;
    pub use super::nvm;
    pub use crate::context::TxContext;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_functionality() {
        // Add tests for core functionality
    }
}
