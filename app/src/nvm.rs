pub mod buffer;
pub mod dkg_keys;

pub use buffer::*;
pub use dkg_keys::*;

#[cfg(feature = "ledger")]
use spin::Mutex;

#[cfg(not(feature = "ledger"))]
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref GLOBAL: Mutex<Option<[u8; 32]>> = Mutex::new(None);
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
    global.take()
}

// Function to get
pub(crate) fn get_tx_hash() -> Option<[u8; 32]> {
    let mut global = GLOBAL.lock();
    global.clone()
}
