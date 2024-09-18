use aead::rand_core::RngCore;

#[cfg(not(feature = "ledger"))]
use rand::SeedableRng;

// Conditionally define LedgerRng
#[cfg(feature = "ledger")]
use ledger_device_sdk::random::LedgerRng as DeviceRng;

pub struct LedgerRng {
    #[cfg(feature = "ledger")]
    rng: DeviceRng,
    #[cfg(not(feature = "ledger"))]
    state: u64,
}

#[cfg(not(feature = "ledger"))]
impl LedgerRng {
    pub fn new() -> Self {
        LedgerRng {
            state: 0x12345678abcdef,
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        // Simple LCG algorithm
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }
}

#[cfg(feature = "ledger")]
impl LedgerRng {
    pub fn new() -> Self {
        LedgerRng { rng: DeviceRng {} }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }
}
