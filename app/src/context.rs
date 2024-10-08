use crate::nvm::buffer::{Buffer, BufferMode};

pub struct TxContext {
    pub buffer: Buffer,
    pub done: bool,
}

// Implement constructor for TxInfo with default values
impl TxContext {
    // Constructor
    pub fn new() -> TxContext {
        TxContext {
            buffer: Buffer::new(),
            done: false,
        }
    }

    pub fn reset_to_receive(&mut self) {
        self.buffer.reset(BufferMode::Receive);
        self.done = false;
    }

    pub fn reset_to_result(&mut self) {
        self.buffer.reset(BufferMode::Result);
        self.done = false;
    }
}

impl Default for TxContext {
    fn default() -> Self {
        Self::new()
    }
}
