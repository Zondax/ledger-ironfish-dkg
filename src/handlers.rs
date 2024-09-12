use crate::{context::TxContext, AppSW, Instruction};
use ledger_device_sdk::io::{ApduHeader, Comm, Event, Reply, StatusWords};

mod dkg_commitments;
mod dkg_get_identity;
mod dkg_get_keys;
mod dkg_get_public_package;
mod dkg_nonces;
mod dkg_round_1;
mod dkg_round_2;
mod dkg_round_3;
mod dkg_sign;
mod get_version;

use dkg_commitments::handler_dkg_commitments;
use dkg_get_identity::handler_dkg_get_identity;
use dkg_get_keys::handler_dkg_get_keys;
use dkg_get_public_package::handler_dkg_get_public_package;
use dkg_nonces::handler_dkg_nonces;
use dkg_round_1::handler_dkg_round_1;
use dkg_round_2::handler_dkg_round_2;
use dkg_round_3::handler_dkg_round_3;
use dkg_sign::handler_dkg_sign;
use get_version::handler_get_version;

pub fn handle_apdu(comm: &mut Comm, ins: &Instruction, ctx: &mut TxContext) -> Result<(), AppSW> {
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
        Instruction::DkgGetKeys { key_type } => handler_dkg_get_keys(comm, key_type),
        Instruction::DkgNonces { chunk } => handler_dkg_nonces(comm, *chunk, ctx),
        Instruction::DkgGetPublicPackage => handler_dkg_get_public_package(comm),
    }
}
