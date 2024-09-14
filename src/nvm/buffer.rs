use crate::AppSW;
use ledger_device_sdk::nvm::*;
use ledger_device_sdk::NVMData;

// This is necessary to store the object in NVM and not in RAM
pub const BUFFER_SIZE: usize = 4000;

#[derive(Clone, Copy)]
pub enum BufferMode {
    Receive,
    Result,
}

#[link_section = ".nvm_data"]
static mut DATA: NVMData<SafeStorage<[u8; BUFFER_SIZE]>> =
    NVMData::new(SafeStorage::new([0u8; BUFFER_SIZE]));

#[derive(Clone, Copy)]
pub struct Buffer {
    pub(crate) pos: usize,
    pub(crate) mode: BufferMode,
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            pos: 0,
            mode: BufferMode::Receive,
        }
    }
}

impl Buffer {
    #[allow(unused)]
    pub fn new() -> Self {
        Buffer::default()
    }

    #[allow(unused)]
    pub fn reset(&mut self, mode: BufferMode) {
        self.pos = 0;
        self.mode = mode;
    }

    #[allow(unused)]
    #[inline(never)]
    pub fn is_valid_write(&self) -> Result<(), AppSW> {
        let buffer = unsafe { DATA.get_mut() };
        if !buffer.is_valid() {
            return Err(AppSW::InvalidNVMWrite);
        }

        Ok(())
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn get_mut_ref(&mut self) -> &mut SafeStorage<[u8; BUFFER_SIZE]> {
        unsafe { DATA.get_mut() }
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn get_element(&self, index: usize) -> Result<u8, AppSW> {
        self.check_read_pos(index)?;

        let buffer = unsafe { DATA.get_mut() };
        Ok(buffer.get_ref()[index])
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn set_element(&self, index: usize, value: u8) -> Result<(), AppSW> {
        self.check_write_pos(index)?;

        let mut updated_data: [u8; BUFFER_SIZE] = unsafe { *DATA.get_mut().get_ref() };
        updated_data[index] = value;
        unsafe {
            DATA.get_mut().update(&updated_data);
        }

        self.is_valid_write()?;
        Ok(())
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn set_slice(&mut self, mut index: usize, value: &[u8]) -> Result<(), AppSW> {
        let mut updated_data: [u8; BUFFER_SIZE] = unsafe { *DATA.get_mut().get_ref() };
        for b in value.iter() {
            self.check_write_pos(index)?;

            updated_data[index] = *b;
            index += 1;
        }
        unsafe {
            DATA.get_mut().update(&updated_data);
        }
        self.is_valid_write()?;

        self.pos += value.len();
        Ok(())
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn get_slice(&self, start_pos: usize, end_pos: usize) -> Result<&[u8], AppSW> {
        self.check_read_pos_slice(end_pos)?;
        let buffer = unsafe { DATA.get_mut() };

        Ok(&buffer.get_ref()[start_pos..end_pos])
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn get_u16(&self, start_pos: usize) -> Result<usize, AppSW> {
        let buffer = unsafe { DATA.get_mut() };
        let buffer_ref = buffer.get_ref();

        self.check_read_pos(start_pos)?;
        self.check_read_pos(start_pos + 1)?;

        Ok(((buffer_ref[start_pos] as u16) << 8 | buffer_ref[start_pos + 1] as u16) as usize)
    }

    fn check_read_pos(&self, index: usize) -> Result<(), AppSW> {
        if index >= self.pos {
            return Err(AppSW::BufferOutOfBounds);
        }

        Ok(())
    }

    fn check_read_pos_slice(&self, index: usize) -> Result<(), AppSW> {
        if index > self.pos {
            return Err(AppSW::BufferOutOfBounds);
        }

        Ok(())
    }

    fn check_write_pos(&self, index: usize) -> Result<(), AppSW> {
        if index >= BUFFER_SIZE {
            return Err(AppSW::BufferOutOfBounds);
        }

        Ok(())
    }
}
