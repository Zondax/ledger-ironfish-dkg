use ledger_device_sdk::io::Comm;
use crate::AppSW;
use crate::nvm::buffer::{Buffer, BUFFER_SIZE};
use crate::context::TxContext;
use crate::utils::zlog_stack;

#[inline(never)]
pub fn accumulate_data(
    comm: &mut Comm,
    chunk: u8,
    ctx: &mut TxContext
) -> Result<(), AppSW> {
    zlog_stack("start accumulate_data\0");

    // Try to get data from comm
    let data = comm.get_data().map_err(|_| AppSW::WrongApduLength)?;

    // First chunk, try to parse the path
    if chunk == 0 {
        // Reset transaction context
        ctx.reset();
        return Ok(());
    // Next chunks, append data to raw_tx and return or parse
    // the transaction if it is the last chunk.
    }

    if ctx.buffer.pos + data.len() > BUFFER_SIZE {
        return Err(AppSW::TxWrongLength);
    }

    // Append data to raw_tx
    ctx.buffer.set_slice(ctx.buffer.pos, data)?;
    ctx.buffer.pos += data.len();

    // If we expect more chunks, return
    if chunk == 1 {
        return Ok(());
    }

    ctx.done = true;
    Ok(())
}