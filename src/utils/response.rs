use crate::AppSW;
use crate::context::TxContext;

const MAX_APDU_SIZE: usize = 253;

#[inline(never)]
pub fn save_result(ctx: &mut TxContext, resp: &[u8]) -> Result<[u8;1], AppSW>{
    ctx.reset();
    ctx.buffer.set_slice(0, resp)?;

    let total_chunks = [((resp.len() + MAX_APDU_SIZE - 1) /MAX_APDU_SIZE) as u8];
    Ok(total_chunks)
}