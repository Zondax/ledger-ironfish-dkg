use crate::bolos::zlog_stack;
use crate::context::TxContext;
use crate::nvm::buffer::BUFFER_SIZE;
use crate::AppSW;
use ledger_device_sdk::io::Comm;

#[inline(never)]
pub fn accumulate_data(comm: &mut Comm, chunk: u8, ctx: &mut TxContext) -> Result<(), AppSW> {
    zlog_stack("start accumulate_data\0");

    // Try to get data from comm
    let data = comm.get_data().map_err(|_| AppSW::WrongApduLength)?;

    // First chunk, try to parse the path
    if chunk == 0 {
        // Reset transaction context
        ctx.reset_to_receive();
        return Ok(());
    }

    if ctx.buffer.pos + data.len() > BUFFER_SIZE {
        return Err(AppSW::TxWrongLength);
    }

    // Append data to raw_tx
    ctx.buffer.set_slice(ctx.buffer.pos, data)?;

    // If we expect more chunks, return
    if chunk == 1 {
        return Ok(());
    }

    ctx.done = true;
    Ok(())
}
