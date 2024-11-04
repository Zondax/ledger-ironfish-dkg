use crate::nvm::buffer::{Buffer, BufferMode};
#[cfg(any(target_os = "stax", target_os = "flex"))]
use ledger_device_sdk::nbgl::NbglHomeAndSettings;

pub struct TxContext {
    pub buffer: Buffer,
    pub done: bool,
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    pub home: NbglHomeAndSettings,
}

// Implement constructor for TxInfo with default values
impl TxContext {
    // Constructor
    pub fn new() -> TxContext {
        TxContext {
            buffer: Buffer::new(),
            done: false,
            #[cfg(any(target_os = "stax", target_os = "flex"))]
            home: Default::default(),
        }
    }

    pub fn reset_to_receive(&mut self) {
        self.buffer.reset(BufferMode::Receive);
        self.done = false;
    }

    pub fn reset_to_result(&mut self) {
        self.buffer.reset(BufferMode::Result);
    }
}

impl Default for TxContext {
    fn default() -> Self {
        Self::new()
    }
}
