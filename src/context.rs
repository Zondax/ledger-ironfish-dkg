use crate::nvm::buffer::Buffer;

pub struct TxContext {
    pub buffer: Buffer,
    pub done: bool
}

// Implement constructor for TxInfo with default values
impl TxContext {
    // Constructor
    pub fn new() -> TxContext {
        TxContext {
            buffer: Buffer::new(),
            done: false
        }
    }

    // Implement reset for TxInfo
    pub fn reset(&mut self) {
        self.buffer.reset();
        self.done = false;
    }
}