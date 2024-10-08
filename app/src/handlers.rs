use crate::{bolos::zlog_stack, context::TxContext, AppSW, Instruction};
use ledger_device_sdk::io::Comm;

mod dkg_backup_keys;
mod dkg_commitments;
mod dkg_get_identitites;
mod dkg_get_identity;
mod dkg_get_keys;
mod dkg_get_public_package;
mod dkg_restore_keys;
mod dkg_round_1;
mod dkg_round_2;
mod dkg_round_3_min;
mod dkg_sign;
mod get_result;
mod get_version;
mod review_tx;

use crate::nvm::buffer::BufferMode;
use crate::nvm::get_and_clear_tx_hash;
use dkg_backup_keys::handler_dkg_backup_keys;
use dkg_commitments::handler_dkg_commitments;
use dkg_get_identitites::handler_dkg_get_identities;
use dkg_get_identity::handler_dkg_get_identity;
use dkg_get_keys::handler_dkg_get_keys;
use dkg_get_public_package::handler_dkg_get_public_package;
use dkg_restore_keys::handler_dkg_restore_keys;
use dkg_round_1::handler_dkg_round_1;
use dkg_round_2::handler_dkg_round_2;
use dkg_round_3_min::handler_dkg_round_3_min;
use dkg_sign::handler_dkg_sign;
use get_result::handler_get_result;
use get_version::handler_get_version;
use review_tx::handler_review_tx;

pub fn handle_apdu(comm: &mut Comm, ins: &Instruction, ctx: &mut TxContext) -> Result<(), AppSW> {
    zlog_stack("handle_apdu\0");

    // If the buffer contains a result from a command, and we receive anything else than GetResult command
    // reset the buffer to receive mode.
    match ins {
        Instruction::GetResult { chunk: _chunk } => {}
        _ => {
            if let BufferMode::Result = ctx.buffer.mode {
                ctx.reset_to_receive();
            }
        }
    };

    // If we receive anything else than DkgSign, DkgCommitments or GetResult command
    // reset the tx_hash ram buffer
    match ins {
        Instruction::DkgSign { chunk: _chunk } => {}
        Instruction::DkgCommitments { chunk: _chunk } => {}
        Instruction::GetResult { chunk: _chunk } => {}
        _ => {
            get_and_clear_tx_hash();
        }
    };

    match ins {
        Instruction::GetVersion => handler_get_version(comm),
        Instruction::DkgGetIdentity { review } => handler_dkg_get_identity(comm, *review),
        Instruction::DkgRound1 { chunk } => handler_dkg_round_1(comm, *chunk, ctx),
        Instruction::DkgRound2 { chunk } => handler_dkg_round_2(comm, *chunk, ctx),
        Instruction::DkgRound3Min { chunk } => handler_dkg_round_3_min(comm, *chunk, ctx),
        Instruction::DkgCommitments { chunk } => handler_dkg_commitments(comm, *chunk, ctx),
        Instruction::DkgSign { chunk } => handler_dkg_sign(comm, *chunk, ctx),
        Instruction::DkgGetKeys { key_type, review } => {
            handler_dkg_get_keys(comm, *review, *key_type)
        }
        Instruction::DkgGetPublicPackage => handler_dkg_get_public_package(comm, ctx),
        Instruction::DkgBackupKeys => handler_dkg_backup_keys(comm, ctx),
        Instruction::DkgRestoreKeys { chunk } => handler_dkg_restore_keys(comm, *chunk, ctx),
        Instruction::GetResult { chunk } => handler_get_result(comm, ctx, *chunk),
        Instruction::ReviewTx { chunk } => handler_review_tx(comm, *chunk, ctx),
        Instruction::DkgGetIdentities => handler_dkg_get_identities(comm, ctx),
    }
}
