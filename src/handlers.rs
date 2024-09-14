use crate::{context::TxContext, AppSW, Instruction};
use ledger_device_sdk::io::{ApduHeader, Comm, Event, Reply, StatusWords};

mod dkg_backup_keys;
mod dkg_commitments;
mod dkg_get_identity;
mod dkg_get_keys;
mod dkg_get_public_package;
mod dkg_restore_keys;
mod dkg_round_1;
mod dkg_round_2;
mod dkg_round_3;
mod dkg_sign;
mod get_result;
mod get_version;

use dkg_backup_keys::handler_dkg_backup_keys;
use dkg_commitments::handler_dkg_commitments;
use dkg_get_identity::handler_dkg_get_identity;
use dkg_get_keys::handler_dkg_get_keys;
use dkg_get_public_package::handler_dkg_get_public_package;
use dkg_restore_keys::handler_dkg_restore_keys;
use dkg_round_1::handler_dkg_round_1;
use dkg_round_2::handler_dkg_round_2;
use dkg_round_3::handler_dkg_round_3;
use dkg_sign::handler_dkg_sign;
use get_result::handler_get_result;
use get_version::handler_get_version;
use crate::nvm::buffer::BufferMode;

pub fn handle_apdu(comm: &mut Comm, ins: &Instruction, ctx: &mut TxContext) -> Result<(), AppSW> {

    // If the buffer contains a result from a command, and we receive anything else than GetResult command
    // reset the buffer to receive mode.
    match ins {
        Instruction::GetResult { chunk: _chunk } => {},
        (_) => {
            if let BufferMode::Result = ctx.buffer.mode{
                ctx.reset_to_receive();
            }
        }
    };

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
        Instruction::DkgGetKeys { key_type } => handler_dkg_get_keys(comm, key_type, ctx),
        Instruction::DkgGetPublicPackage => handler_dkg_get_public_package(comm, ctx),
        Instruction::DkgBackupKeys => handler_dkg_backup_keys(comm, ctx),
        Instruction::DkgRestoreKeys { chunk } => handler_dkg_restore_keys(comm, *chunk, ctx),
        Instruction::GetResult { chunk } => handler_get_result(comm, ctx, *chunk),
    }
}
