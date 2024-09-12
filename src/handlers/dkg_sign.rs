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
use ironfish_frost::{frost::SigningPackage, frost::Randomizer};
use ironfish_frost::frost::keys::KeyPackage;
use ironfish_frost::frost::round1::SigningNonces;
use ironfish_frost::frost::round2;
use ledger_device_sdk::io::{Comm, Event};
use crate::accumulator::accumulate_data;
use crate::nvm::buffer::{Buffer};
use crate::context::TxContext;
use crate::utils::{zlog_stack};
use crate::nvm::dkg_keys::DkgKeys;

const MAX_APDU_SIZE: usize = 253;

#[inline(never)]
pub fn handler_dkg_sign(
    comm: &mut Comm,
    chunk: u8,
    ctx: &mut TxContext,
) -> Result<(), AppSW> {
    zlog_stack("start handler_dkg_sign\0");

    accumulate_data(comm, chunk, ctx)?;
    if !ctx.done {
        return Ok(());
    }

    let (frost_signing_package, nonces, randomizer) = parse_tx(&ctx.buffer)?;
    let key_package = DkgKeys.load_key_package()?;

    zlog_stack("start signing\0");
    let signature = round2::sign(
        &frost_signing_package,
        &nonces,
        &key_package,
        randomizer,
    );

    zlog_stack("unwrap sig result\0");
    let sig = signature.unwrap().serialize();

    send_apdu_chunks(comm, sig)
}

#[inline(never)]
fn parse_tx(buffer: &Buffer) -> Result<(SigningPackage, SigningNonces, Randomizer), AppSW>{
    zlog_stack("start parse_tx\0");

    let mut tx_pos = 0;

    let pk_randomness_len = buffer.get_u16(tx_pos)?;
    tx_pos +=2;

    let data = buffer.get_slice(tx_pos,tx_pos+pk_randomness_len)?;
    let randomizer = Randomizer::deserialize(data).map_err(|_| AppSW::InvalidRandomizer)?;
    tx_pos +=pk_randomness_len;

    let frost_signing_package_len = buffer.get_u16(tx_pos)?;
    tx_pos +=2;

    let data = buffer.get_slice(tx_pos,tx_pos+frost_signing_package_len)?;
    let frost_signing_package = SigningPackage::deserialize(data).map_err(|_| AppSW::InvalidSigningPackage)?;
    tx_pos += frost_signing_package_len;

    let nonces_len = buffer.get_u16(tx_pos)?;
    tx_pos +=2;

    let data = buffer.get_slice(tx_pos,tx_pos+nonces_len)?;
    let nonces = SigningNonces::deserialize(data).map_err(|_| AppSW::InvalidSigningNonces)?;
    tx_pos += nonces_len;

    if tx_pos != buffer.pos {
        return Err(AppSW::InvalidPayload);
    }

    Ok((frost_signing_package, nonces, randomizer))
}

#[inline(never)]
fn send_apdu_chunks(comm: &mut Comm, data_vec: Vec<u8>) -> Result<(), AppSW> {
    zlog_stack("start send_apdu_chunks\0");

    let data = data_vec.as_slice();
    let total_chunks = (data.len() + MAX_APDU_SIZE - 1) / MAX_APDU_SIZE;

    for (i, chunk) in data.chunks(MAX_APDU_SIZE).enumerate() {
        comm.append(chunk);

        if i < total_chunks - 1 {
            comm.reply_ok();
            match comm.next_event() {
                Event::Command(Instruction::DkgSign {chunk: 0}) => {}
                _ => {},
            }
        }
    }

    Ok(())
}
