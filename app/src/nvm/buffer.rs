use crate::AppSW;
use ledger_device_sdk::nvm::*;
use ledger_device_sdk::NVMData;
use nom::number::complete::be_u16;

// This is necessary to store the object in NVM and not in RAM
// The max data received is round2 for 4 participants, which sends 2250 bytes.
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
        let buffer = unsafe { DATA.get_mut() };
        buffer
            .get_ref()
            .get(index)
            .ok_or(AppSW::BufferOutOfBounds)
            .copied()
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn set_element(&self, index: usize, value: u8) -> Result<(), AppSW> {
        let mut updated_data: [u8; BUFFER_SIZE] = unsafe { *DATA.get_mut().get_ref() };

        updated_data
            .get_mut(index)
            .map(|v| *v = value)
            .ok_or(AppSW::BufferOutOfBounds)?;

        unsafe {
            DATA.get_mut().update(&updated_data);
        }

        self.is_valid_write()?;
        Ok(())
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn set_slice(&mut self, index: usize, value: &[u8]) -> Result<(), AppSW> {
        let end_index = index + value.len();
        self.check_write_pos(end_index - 1)?;

        let mut updated_data: [u8; BUFFER_SIZE] = unsafe { *DATA.get_mut().get_ref() };

        // Copy the entire slice at once
        updated_data[index..end_index].copy_from_slice(value);

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

        // Check we are within the read section of the internal buffer
        self.check_read_pos(start_pos + 1)?;

        let input = &buffer_ref[start_pos..];
        let (_, value) = be_u16(input)?;

        Ok(value as usize)
    }

    pub fn get_full_buffer(&self) -> &[u8] {
        let buffer = unsafe { DATA.get_mut() };
        buffer.get_ref().as_slice()
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
