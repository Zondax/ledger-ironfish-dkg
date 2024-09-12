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

use crate::{AppSW};
use alloc::vec::Vec;
use ironfish_frost::dkg;
use ironfish_frost::dkg::group_key::GroupSecretKey;
use ironfish_frost::frost::keys::PublicKeyPackage as FrostPublicKeyPackage;
use ironfish_frost::error::IronfishFrostError;
use ironfish_frost::frost::keys::KeyPackage;
use ledger_device_sdk::io::{Comm};
use crate::accumulator::accumulate_data;
use crate::nvm::buffer::{Buffer};
use crate::context::TxContext;
use crate::handlers::dkg_get_identity::compute_dkg_secret;
use crate::nvm::dkg_keys::DkgKeys;
use crate::utils::{zlog_stack};


pub struct MinTx {
    identity_index: u8,
    round_1_packages: Vec<Vec<u8>>,
    round_2_packages: Vec<Vec<u8>>,
    round_2_secret_package: Vec<u8>,
    participants: Vec<Vec<u8>>,
    gsk_bytes: Vec<Vec<u8>>,
}

#[inline(never)]
pub fn handler_dkg_round_3(
    comm: &mut Comm,
    chunk: u8,
    ctx: &mut TxContext,
) -> Result<(), AppSW> {
    zlog_stack("start handler_dkg_round_3\0");

    accumulate_data(comm, chunk, ctx)?;
    if !ctx.done {
        return Ok(());
    }

    // Try to deserialize the transaction
    let min_tx = parse_tx_min(&ctx.buffer)?;

    let (key_package, public_key_package, group_secret_key)
        = compute_dkg_round_3_min(&min_tx).map_err(|_| AppSW::DkgRound3Fail)?;
    drop(min_tx);

    DkgKeys.save_keys(key_package, public_key_package, group_secret_key);

    Ok(())
}

#[inline(never)]
fn parse_tx_min(buffer: &Buffer) -> Result<MinTx, AppSW>{
    zlog_stack("start parse_tx_min round3\0");

    let mut tx_pos:usize = 0;

    let identity_index = buffer.get_element(tx_pos)?;
    tx_pos +=1;

    // Round 1 public packages
    let elements = buffer.get_element(tx_pos)?;
    tx_pos +=1;

    let len = buffer.get_u16(tx_pos)?;
    tx_pos +=2;

    let mut round_1_packages = Vec::with_capacity(elements as usize);
    for _i in 0..elements {
        zlog_stack("start parse_round_1 - e\0");
        let package = buffer.get_slice(tx_pos,tx_pos+len)?;
        tx_pos += len;

        zlog_stack("push parse_round_1 - e\0");
        round_1_packages.push(package.to_vec());
        zlog_stack("done parse_round_1 - e\0");
    }

    // Round 2 public packages
    let elements = buffer.get_element(tx_pos)?;
    tx_pos +=1;

    let len = buffer.get_u16(tx_pos)?;
    tx_pos +=2;

    let mut round_2_packages = Vec::with_capacity(elements as usize);
    for _i in 0..elements {
        zlog_stack("start parse_round_2 - e\0");
        let r2_package = buffer.get_slice(tx_pos,tx_pos+len)?;
        tx_pos += len;

        zlog_stack("push parse_round_2 - e\0");
        round_2_packages.push(r2_package.to_vec());
        zlog_stack("done parse_round_2 - e\0");
    }

    // round 2 secret pkg
    let len = buffer.get_u16(tx_pos)?;
    tx_pos +=2;

    let round_2_secret_package_slice = buffer.get_slice(tx_pos,tx_pos+len)?;
    let round_2_secret_package = round_2_secret_package_slice.to_vec();
    tx_pos += len;

    // participants
    let elements = buffer.get_element(tx_pos)?;
    tx_pos +=1;

    let len = buffer.get_u16(tx_pos)?;
    tx_pos +=2;

    let mut participants = Vec::with_capacity(elements as usize);
    for _i in 0..elements {
        zlog_stack("start parse participants - e\0");
        let participant = buffer.get_slice(tx_pos,tx_pos+len)?;
        tx_pos += len;

        zlog_stack("push parse participants - e\0");
        participants.push(participant.to_vec());
        zlog_stack("done parse participants - e\0");
    }

    // gsk bytes
    let elements = buffer.get_element(tx_pos)?;
    tx_pos +=1;

    let len = buffer.get_u16(tx_pos)?;
    tx_pos +=2;

    let mut gsk_bytes = Vec::with_capacity(elements as usize);
    for _i in 0..elements {
        zlog_stack("start parse gsk - e\0");
        let gsk = buffer.get_slice(tx_pos,tx_pos+len)?;
        tx_pos += len;

        zlog_stack("push parse sgk - e\0");
        gsk_bytes.push(gsk.to_vec());
        zlog_stack("done parse gsk - e\0");
    }

    if tx_pos != buffer.pos {
        return Err(AppSW::InvalidPayload);
    }

    zlog_stack("done parse_tx round3_min\0");

    Ok(MinTx{
        round_2_secret_package,
        round_1_packages,
        round_2_packages,
        identity_index,
        participants,
        gsk_bytes,
    })
}

#[inline(never)]
fn compute_dkg_round_3_min(min_tx: &MinTx) -> Result<(KeyPackage, FrostPublicKeyPackage, GroupSecretKey), IronfishFrostError> {
    zlog_stack("start compute_dkg_round_3\0");

    let secret = compute_dkg_secret(min_tx.identity_index);

    let p = min_tx.participants.iter().map(|p| p.as_slice()).collect();
    let r1 = min_tx.round_1_packages.iter().map(|r| r.as_slice()).collect();
    let r2 = min_tx.round_2_packages.iter().map(|r| r.as_slice()).collect();
    let gsk = min_tx.gsk_bytes.iter().map(|g| g.as_slice()).collect();

    dkg::round3::round3_min(
        &secret,
        p,
        &min_tx.round_2_secret_package,
        r1,
        r2,
        gsk,
    )
}