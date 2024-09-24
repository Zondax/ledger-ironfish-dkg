pub mod buffer;
pub mod dkg_keys;

pub use buffer::*;
pub use dkg_keys::*;

#[cfg(feature = "ledger")]
use spin::Mutex;

#[cfg(not(feature = "ledger"))]
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref GLOBAL: Mutex<Option<[u8; 32]>> = Mutex::new(Some([0u8; 32]));
}

// not sure if this is the best place,
// Function to set the global array
pub(crate) fn set_tx_hash(data: [u8; 32]) {
    let mut global = GLOBAL.lock();
    global.replace(data);
}

// Function to get and clear the global array
pub(crate) fn get_and_clear_tx_hash() -> Option<[u8; 32]> {
    let mut global = GLOBAL.lock();

    // Take the current value, and replace it with zeros
    let value = global.replace([0; 32]);
    // Once memory is zero-ed, let's set it to None
    global.take();

    // And return the value found in the first place
    value
}

// Function to get
pub(crate) fn get_tx_hash() -> Option<[u8; 32]> {
    let global = GLOBAL.lock();
    global.clone()
}
