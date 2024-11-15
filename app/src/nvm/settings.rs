use ledger_device_sdk::nvm::*;
use ledger_device_sdk::NVMData;

const OFF_STATE: u8 = 0;
const ON_STATE: u8 = 1;

const SETTINGS_SIZE: usize = 10;
const EXPERT_MODE_FLAG: usize = 0;

#[link_section = ".nvm_data"]
static mut DATA: NVMData<AtomicStorage<[u8; SETTINGS_SIZE]>> =
    NVMData::new(AtomicStorage::new(&[0u8; SETTINGS_SIZE]));

#[derive(Clone, Copy)]
pub struct Settings;

impl Default for Settings {
    fn default() -> Self {
        Settings
    }
}

impl Settings {
    #[inline(never)]
    pub fn get_mut(&mut self) -> &mut AtomicStorage<[u8; SETTINGS_SIZE]> {
        return unsafe { DATA.get_mut() };
    }

    #[inline(never)]
    pub fn get_ref(&mut self) -> &AtomicStorage<[u8; SETTINGS_SIZE]> {
        return unsafe { DATA.get_ref() };
    }

    pub fn get_element(&self, index: usize) -> Option<u8> {
        let storage = unsafe { DATA.get_ref() };
        let settings = storage.get_ref();
        settings.get(index).copied()
    }

    pub fn set_element(&self, index: usize, value: u8) {
        if index >= SETTINGS_SIZE {
            return;
        }
        let storage = unsafe { DATA.get_mut() };
        let mut updated_data = *storage.get_ref();
        updated_data[index] = value;
        storage.update(&updated_data);
    }

    pub fn app_expert_mode(&self) -> bool {
        self.get_element(EXPERT_MODE_FLAG)
            .map(|mode| mode == ON_STATE)
            .unwrap_or(false)
    }

    pub fn toggle_expert_mode(&self) {
        match self.get_element(EXPERT_MODE_FLAG) {
            Some(OFF_STATE) => Settings.set_element(EXPERT_MODE_FLAG, ON_STATE),
            _ => Settings.set_element(EXPERT_MODE_FLAG, OFF_STATE),
        }
    }
}
