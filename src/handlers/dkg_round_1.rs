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

use crate::accumulator::accumulate_data;
use crate::bolos::{zlog, zlog_stack};
use crate::context::TxContext;
use crate::handlers::dkg_get_identity::compute_dkg_secret;
use crate::nvm::buffer::Buffer;
use crate::nvm::dkg_keys::DkgKeys;
use crate::utils::response::save_result;
use crate::AppSW;
use alloc::vec::Vec;
use ironfish_frost::dkg;
use ironfish_frost::participant::{Identity, Secret};
use ledger_device_sdk::io::Comm;
use ledger_device_sdk::random::LedgerRng;
use crate::app_ui::generic::ui_run_action;

const IDENTITY_LEN: usize = 129;

pub struct Tx {
    identity_index: u8,
    identities: Vec<Identity>,
    min_signers: u8,
}

#[inline(never)]
pub fn handler_dkg_round_1(comm: &mut Comm, chunk: u8, ctx: &mut TxContext) -> Result<(), AppSW> {
    zlog_stack("start handler_dkg_round_1\0");

    accumulate_data(comm, chunk, ctx)?;
    if !ctx.done {
        return Ok(());
    }

    let mut tx: Tx = parse_tx(&ctx.buffer)?;

    if !ui_run_action(&["Run DKG Round 1?"]){
        return Err(AppSW::Deny);
    }

    let dkg_secret = compute_dkg_secret(tx.identity_index);

    let resp = compute_dkg_round_1(comm, &dkg_secret, &mut tx)?;

    DkgKeys.save_round_1_data(&tx.identities, tx.min_signers)?;

    let total_chunks = save_result(ctx, resp.as_slice())?;
    comm.append(&total_chunks);
    Ok(())
}

fn parse_tx(buffer: &Buffer) -> Result<Tx, AppSW> {
    let mut tx_pos: usize = 0;

    let identity_index = buffer.get_element(tx_pos)?;
    tx_pos += 1;

    let elements = buffer.get_element(tx_pos)?;
    tx_pos += 1;

    let mut identities: Vec<Identity> = Vec::with_capacity(elements as usize);
    for _i in 0..elements {
        let data = buffer.get_slice(tx_pos, tx_pos + IDENTITY_LEN)?;
        let identity = Identity::deserialize_from(data).map_err(|_| AppSW::InvalidIdentity)?;
        tx_pos += IDENTITY_LEN;

        identities.push(identity);
    }

    let min_signers = buffer.get_element(tx_pos)?;
    tx_pos += 1;

    if tx_pos != buffer.pos {
        return Err(AppSW::InvalidPayload);
    }

    Ok(Tx {
        identities,
        min_signers,
        identity_index,
    })
}

fn compute_dkg_round_1(comm: &mut Comm, secret: &Secret, tx: &mut Tx) -> Result<Vec<u8>, AppSW> {
    zlog("start compute_dkg_round_1\n\0");

    let mut rng = LedgerRng {};

    let (mut round1_secret_package_vec, round1_public_package) = dkg::round1::round1(
        &secret.to_identity(),
        tx.min_signers as u16,
        &tx.identities,
        &mut rng,
    )
    .unwrap();

    let mut resp: Vec<u8> = Vec::new();
    let mut round1_public_package_vec = round1_public_package.serialize();
    let round1_public_package_len = round1_public_package_vec.len();
    let round1_secret_package_len = round1_secret_package_vec.len();

    resp.append(
        &mut [
            (round1_secret_package_len >> 8) as u8,
            (round1_secret_package_len & 0xFF) as u8,
        ]
        .to_vec(),
    );
    resp.append(&mut round1_secret_package_vec);
    resp.append(
        &mut [
            (round1_public_package_len >> 8) as u8,
            (round1_public_package_len & 0xFF) as u8,
        ]
        .to_vec(),
    );
    resp.append(&mut round1_public_package_vec);

    Ok(resp)
}
