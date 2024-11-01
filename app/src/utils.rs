mod bip32;
#[cfg(feature = "ledger")]
pub mod response;
#[macro_use]
pub mod int_format;

use crate::bolos::zlog_stack;
use alloc::string::{String, ToString};
pub use bip32::Bip32Path;
use core::cmp;

pub fn str_to_array<const SIZE: usize>(string: &str) -> [u8; SIZE] {
    let bytes = string.as_bytes();
    let num_to_copy = cmp::min(bytes.len(), SIZE);

    let mut arr = [0u8; SIZE];
    arr[..num_to_copy].copy_from_slice(&bytes[..num_to_copy]);

    arr
}

#[inline(never)]
pub fn int_to_str(num: u8) -> String {
    use lexical_core::BUFFER_SIZE as INT_BUFFER_SIZE;

    zlog_stack("start int_to_str\0");
    let mut buffer = [b'0'; INT_BUFFER_SIZE];
    let raw = lexical_core::write(num, &mut buffer);
    let num_str = core::str::from_utf8(raw).unwrap();
    zlog_stack("after int_to_str\0");

    num_str.to_string()
}
