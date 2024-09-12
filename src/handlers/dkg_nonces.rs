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

use crate::{AppSW, Instruction};
use alloc::vec::Vec;
use ironfish_frost::frost::keys::KeyPackage;
use ledger_device_sdk::io::{Comm, Event};
use serde::Serialize;
use crate::accumulator::accumulate_data;
use crate::nvm::buffer::{Buffer};
use crate::context::TxContext;
use crate::utils::{zlog_stack};
use ironfish_frost::nonces::deterministic_signing_nonces;
use ironfish_frost::participant::Identity;
use crate::nvm::dkg_keys::DkgKeys;

const MAX_APDU_SIZE: usize = 253;
const IDENTITY_LEN: usize = 129;
const TX_HASH_LEN: usize = 32;

#[inline(never)]
pub fn handler_dkg_nonces(
    comm: &mut Comm,
    chunk: u8,
    ctx: &mut TxContext,
) -> Result<(), AppSW> {
    zlog_stack("start handler_dkg_nonces\0");

    accumulate_data(comm, chunk, ctx)?;
    if !ctx.done {
        return Ok(());
    }

    let (identities, tx_hash) = parse_tx(&ctx.buffer)?;
    let key_package = DkgKeys.load_key_package()?;

    let nonces = deterministic_signing_nonces(
        key_package.signing_share(),
        tx_hash,
        &identities,
    );

    let ser = nonces.serialize().unwrap();

    send_apdu_chunks(comm, ser)
}

#[inline(never)]
fn parse_tx(buffer: &Buffer) -> Result<(Vec<Identity>, &[u8]), AppSW>{
    zlog_stack("start parse_tx\0");

    let mut tx_pos = 0;
    let elements = buffer.get_element(tx_pos)?;
    tx_pos +=1;

    let mut identities:Vec<Identity> = Vec::with_capacity(elements as usize);
    for _i in 0..elements {
        let data = buffer.get_slice(tx_pos,tx_pos+IDENTITY_LEN)?;
        let identity = Identity::deserialize_from(data).map_err(|_| AppSW::InvalidIdentity)?;
        tx_pos += IDENTITY_LEN;

        identities.push(identity);
    }


    let tx_hash = buffer.get_slice(tx_pos, tx_pos + TX_HASH_LEN)?;
    tx_pos += TX_HASH_LEN;

    if tx_pos != buffer.pos {
        return Err(AppSW::InvalidPayload);
    }

    Ok((identities, tx_hash))
}

#[inline(never)]
fn send_apdu_chunks(comm: &mut Comm, data_vec: Vec<u8>) -> Result<(), AppSW> {
    zlog_stack("start send_apdu_chunks\0");

    let data = data_vec.as_slice();
    let total_chunks = (data.len() + MAX_APDU_SIZE - 1) / MAX_APDU_SIZE;

    for (i, chunk) in data.chunks(MAX_APDU_SIZE).enumerate() {
        zlog_stack("iter send_apdu_chunks\0");
        comm.append(chunk);

        if i < total_chunks - 1 {
            zlog_stack("another send_apdu_chunks\0");
            comm.reply_ok();
            match comm.next_event() {
                Event::Command(Instruction::DkgNonces {chunk: 0}) => {}
                _ => {},
            }
        }
    }

    Ok(())
}
