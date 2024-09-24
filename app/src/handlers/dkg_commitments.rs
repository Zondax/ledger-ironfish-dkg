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
use crate::bolos::zlog_stack;
use crate::context::TxContext;
use crate::ironfish::constants::TX_HASH_LEN;
use crate::nvm::buffer::Buffer;
use crate::nvm::dkg_keys::DkgKeys;
use crate::utils::response::save_result;
use crate::AppSW;
use ironfish_frost::frost::round1::SigningCommitments;
use ironfish_frost::nonces::deterministic_signing_nonces;
use ledger_device_sdk::io::Comm;

#[inline(never)]
pub fn handler_dkg_commitments(
    comm: &mut Comm,
    chunk: u8,
    ctx: &mut TxContext,
) -> Result<(), AppSW> {
    zlog_stack("start handler_dkg_commitments\0");

    accumulate_data(comm, chunk, ctx)?;
    if !ctx.done {
        return Ok(());
    }

    let tx_hash = parse_tx(&ctx.buffer)?;

    let key_package = DkgKeys.load_key_package()?;
    let identities = DkgKeys.load_identities()?;

    let nonces = deterministic_signing_nonces(key_package.signing_share(), tx_hash, &identities);

    let signing_commitment: SigningCommitments = (&nonces).into();
    let resp = signing_commitment.serialize().unwrap();

    let total_chunks = save_result(ctx, resp.as_slice())?;
    comm.append(&total_chunks);
    Ok(())
}

#[inline(never)]
fn parse_tx(buffer: &Buffer) -> Result<&[u8], AppSW> {
    zlog_stack("start parse_tx\0");

    let mut tx_pos = 0;
    let tx_hash = buffer.get_slice(tx_pos, tx_pos + TX_HASH_LEN)?;
    tx_pos += TX_HASH_LEN;

    if tx_pos != buffer.pos {
        return Err(AppSW::InvalidPayload);
    }

    Ok(tx_hash)
}
