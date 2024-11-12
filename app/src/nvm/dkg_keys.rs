use crate::bolos::zlog_stack;
use crate::crypto::{GroupSecretKeyGuard, KeyPackageGuard};
use crate::AppSW;
use alloc::vec::Vec;
use ironfish_frost::dkg::group_key::GroupSecretKey;
use ironfish_frost::frost::keys::KeyPackage;
use ironfish_frost::frost::keys::PublicKeyPackage as FrostPublicKeyPackage;
use ironfish_frost::participant::{Identity, IDENTITY_LEN};
use ledger_device_sdk::nvm::*;
use ledger_device_sdk::NVMData;

// This is necessary to store the object in NVM and not in RAM
// The max data saved here is for 4 participants, which sends 1024 bytes.
pub const DKG_KEYS_MAX_SIZE: usize = 1300;

// Fix positions with u8 values
const DKG_STATUS: usize = 0;
const DKG_VERSION: usize = 1;
const MIN_SIGNERS_POS: usize = 2;
const IDENTITY_INDEX_POS: usize = 3;

// Leave some bytes free for future u8 we want to save something new... positions from 0 to 9

// Positions (u16) to indicate where these entities are located in the buffer
// On each entity, the first 2 bytes are the length of it, followed by its data
const IDENTITIES_POS: usize = 10;
const KEY_PACKAGE_POS: usize = 12;
const GROUP_KEY_PACKAGE_POS: usize = 14;
const FROST_PUBLIC_PACKAGE_POS: usize = 16;

// Again, leave some bytes free for future entities (u16) we want to save something new... positions from 10 to 24 (7 entities, as 2 bytes per each)
const DATA_STARTING_POS: u16 = 24;

pub enum DkgKeyStatus {
    Idle,
    Initiated,
    Completed,
}

pub enum DkgKeyVersion {
    V1 = 1,
}

#[link_section = ".nvm_data"]
static mut DATA: NVMData<SafeStorage<[u8; DKG_KEYS_MAX_SIZE]>> =
    NVMData::new(SafeStorage::new([0u8; DKG_KEYS_MAX_SIZE]));

#[derive(Clone, Copy)]
pub struct DkgKeys;

impl Default for DkgKeys {
    fn default() -> Self {
        DkgKeys
    }
}

impl DkgKeys {
    pub fn is_valid_write(&self) -> Result<(), AppSW> {
        let buffer = unsafe { DATA.get_mut() };
        if !buffer.is_valid() {
            return Err(AppSW::InvalidNVMWrite);
        }

        Ok(())
    }

    #[inline(never)]
    pub fn get_element(&self, index: usize) -> Option<u8> {
        let buffer_ref: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };
        buffer_ref.get(index).copied()
    }

    #[inline(never)]
    pub fn get_u16(&self, start_pos: usize) -> Option<usize> {
        let buffer_ref: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };
        let bytes = buffer_ref[start_pos..start_pos + 2].try_into().ok()?;
        Some(u16::from_be_bytes(bytes) as usize)
    }

    #[inline(never)]
    pub fn set_element(&self, index: usize, value: u8) -> Result<(), AppSW> {
        let mut updated_data: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };

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
    pub fn set_slice(&self, index: usize, value: &[u8]) -> Result<(), AppSW> {
        let end_index = index + value.len();
        self.check_write_pos(end_index - 1)?; // Check only the last position

        let mut updated_data: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };

        updated_data[index..end_index].copy_from_slice(value);

        unsafe {
            DATA.get_mut().update(&updated_data);
        }
        self.is_valid_write()?;
        Ok(())
    }

    #[inline(never)]
    pub fn set_slice_with_len(&self, index: usize, value: &[u8]) -> Result<usize, AppSW> {
        let len = value.len();

        let total_len = 2 + len; // 2 bytes for length + actual data length
        self.check_write_pos(index + total_len - 1)?; // -1 to offset correctly the length and pos

        let mut updated_data: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };

        // Write length as big-endian u16
        updated_data[index..index + 2].copy_from_slice(&(len as u16).to_be_bytes());

        // Write the actual data
        updated_data[index + 2..index + total_len].copy_from_slice(value);

        unsafe {
            DATA.get_mut().update(&updated_data);
        }
        self.is_valid_write()?;

        Ok(index + total_len)
    }

    #[inline(never)]
    pub fn set_u16(&self, index: usize, value: u16) -> Result<usize, AppSW> {
        self.check_write_pos(index + 1)?; // Check only the last position

        let mut updated_data: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };

        // Convert u16 to big-endian bytes and copy them
        updated_data
            .get_mut(index..index + 2)
            .map(|slice| slice.copy_from_slice(&value.to_be_bytes()))
            .ok_or(AppSW::BufferOutOfBounds)?;

        unsafe {
            DATA.get_mut().update(&updated_data);
        }

        self.is_valid_write()?;
        Ok(index + 2)
    }

    fn check_write_pos(&self, index: usize) -> Result<(), AppSW> {
        if index >= DKG_KEYS_MAX_SIZE {
            return Err(AppSW::BufferOutOfBounds);
        }

        Ok(())
    }

    #[inline(never)]
    pub fn save_round_1_data(
        &self,
        identities: &Vec<Identity>,
        min_signers: u8,
        identity_index: u8,
    ) -> Result<(), AppSW> {
        zlog_stack("start save_round_1_data\0");

        self.set_u16(IDENTITIES_POS, DATA_STARTING_POS)?;

        let mut pos = DATA_STARTING_POS as usize;
        self.set_u16(pos, (identities.len() * IDENTITY_LEN) as u16)?;
        pos += 2;

        for i in identities.iter() {
            let slice = i.serialize();
            self.set_slice(pos, slice.as_slice())?;
            pos += IDENTITY_LEN;
        }

        self.set_element(MIN_SIGNERS_POS, min_signers)?;
        self.set_element(IDENTITY_INDEX_POS, identity_index)?;

        self.update_keys_status(DkgKeyStatus::Initiated, DkgKeyVersion::V1)
    }

    #[inline(never)]
    pub fn update_keys_status(
        &self,
        status: DkgKeyStatus,
        version: DkgKeyVersion,
    ) -> Result<(), AppSW> {
        zlog_stack("start update_keys_status\0");

        match version {
            DkgKeyVersion::V1 => self.set_element(DKG_VERSION, 1),
        }?;

        match status {
            DkgKeyStatus::Idle => self.set_element(DKG_STATUS, 0),
            DkgKeyStatus::Initiated => self.set_element(DKG_STATUS, 1),
            DkgKeyStatus::Completed => self.set_element(DKG_STATUS, 2),
        }
    }

    #[inline(never)]
    pub fn save_keys(
        &mut self,
        key_package: &KeyPackage,
        public_key_package: FrostPublicKeyPackage,
        group_secret_key: &GroupSecretKey,
    ) -> Result<(), AppSW> {
        zlog_stack("start save_keys\0");

        let status = self.get_keys_status()?;
        match status {
            DkgKeyStatus::Initiated => {}
            _ => {
                return Err(AppSW::InvalidDkgStatus);
            }
        }

        // Read where the previous data end up
        let identities_pos: usize = self
            .get_u16(IDENTITIES_POS)
            .ok_or(AppSW::InvalidDkgStatus)?;

        let identities_len: usize = self
            .get_u16(identities_pos)
            .ok_or(AppSW::InvalidDkgStatus)?;
        let mut pos = identities_pos + 2 + identities_len;

        self.set_u16(KEY_PACKAGE_POS, pos as u16)?;
        pos = self.set_slice_with_len(pos, key_package.serialize().unwrap().as_slice())?;
        self.set_u16(GROUP_KEY_PACKAGE_POS, pos as u16)?;
        pos = self.set_slice_with_len(pos, group_secret_key.as_slice())?;
        self.set_u16(FROST_PUBLIC_PACKAGE_POS, pos as u16)?;
        self.set_slice_with_len(pos, public_key_package.serialize().unwrap().as_slice())?;

        self.update_keys_status(DkgKeyStatus::Completed, DkgKeyVersion::V1)
    }

    #[inline(never)]
    pub fn restore_keys(&self, data: &[u8]) -> Result<(), AppSW> {
        zlog_stack("start restore_keys\0");

        if data[DKG_VERSION] != 1 {
            return Err(AppSW::InvalidDkgKeysVersion);
        }

        self.set_slice(0, data)
    }

    #[inline(never)]
    pub fn get_keys_status(&mut self) -> Result<DkgKeyStatus, AppSW> {
        let buffer_ref: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };

        DkgKeysReader::get_keys_status(&buffer_ref)
    }

    #[inline(never)]
    pub fn backup_keys(&self) -> Result<Vec<u8>, AppSW> {
        let buffer_ref: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };

        DkgKeysReader::backup_keys(&buffer_ref)
    }

    #[inline(never)]
    pub fn load_group_secret_key(&mut self) -> Result<GroupSecretKeyGuard, AppSW> {
        let buffer_ref: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };

        DkgKeysReader::load_group_secret_key(&buffer_ref)
    }

    #[inline(never)]
    pub fn load_frost_public_key_package(&mut self) -> Result<FrostPublicKeyPackage, AppSW> {
        let buffer_ref: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };

        DkgKeysReader::load_frost_public_key_package(&buffer_ref)
    }

    #[inline(never)]
    pub fn load_key_package(&mut self) -> Result<KeyPackageGuard, AppSW> {
        let buffer_ref: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };

        DkgKeysReader::load_key_package(&buffer_ref)
    }

    #[inline(never)]
    pub fn load_min_signers(&mut self) -> Result<usize, AppSW> {
        let buffer_ref: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };

        DkgKeysReader::load_min_signers(&buffer_ref)
    }

    #[inline(never)]
    pub fn load_identity_index(&mut self) -> Result<usize, AppSW> {
        let buffer_ref: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };

        DkgKeysReader::load_identity_index(&buffer_ref)
    }

    #[inline(never)]
    pub fn load_identities(&mut self) -> Result<Vec<Identity>, AppSW> {
        let buffer_ref: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };

        DkgKeysReader::load_identities(&buffer_ref)
    }
}

// Allows to read and parse dkg keys data from a buffer
// It does not matter where that buffer come from
pub struct DkgKeysReader;

impl DkgKeysReader {
    #[inline(never)]
    pub fn get_element(data: &[u8], index: usize) -> u8 {
        data[index]
    }

    #[inline(never)]
    pub fn get_slice(data: &[u8], start_pos: usize, end_pos: usize) -> &[u8] {
        &data[start_pos..end_pos]
    }

    #[inline(never)]
    pub fn get_u16(data: &[u8], start_pos: usize) -> usize {
        ((data[start_pos] as u16) << 8 | data[start_pos + 1] as u16) as usize
    }

    #[inline(never)]
    pub fn get_keys_status(data: &[u8]) -> Result<DkgKeyStatus, AppSW> {
        zlog_stack("start get_keys_status\0");

        let status = DkgKeysReader::get_element(data, DKG_STATUS);
        match status {
            0 => Ok(DkgKeyStatus::Idle),
            1 => Ok(DkgKeyStatus::Initiated),
            2 => Ok(DkgKeyStatus::Completed),
            _ => Err(AppSW::InvalidDkgStatus),
        }
    }

    #[inline(never)]
    pub fn load_group_secret_key(data: &[u8]) -> Result<GroupSecretKeyGuard, AppSW> {
        zlog_stack("start load_group_secret_key\0");

        let status = DkgKeysReader::get_keys_status(data)?;
        match status {
            DkgKeyStatus::Completed => {}
            _ => {
                return Err(AppSW::InvalidDkgStatus);
            }
        }

        let mut start = DkgKeysReader::get_u16(data, GROUP_KEY_PACKAGE_POS);
        let len = DkgKeysReader::get_u16(data, start);
        start += 2;

        let raw = DkgKeysReader::get_slice(data, start, start + len);
        let parsed = GroupSecretKeyGuard::from_raw(raw)?;

        Ok(parsed)
    }

    #[inline(never)]
    pub fn load_frost_public_key_package(data: &[u8]) -> Result<FrostPublicKeyPackage, AppSW> {
        zlog_stack("start load_frost_public_key_package\0");

        let status = DkgKeysReader::get_keys_status(data)?;
        match status {
            DkgKeyStatus::Completed => {}
            _ => {
                return Err(AppSW::InvalidDkgStatus);
            }
        }

        let mut start = DkgKeysReader::get_u16(data, FROST_PUBLIC_PACKAGE_POS);
        let len = DkgKeysReader::get_u16(data, start);
        start += 2;

        let data = DkgKeysReader::get_slice(data, start, start + len);
        let parsed =
            FrostPublicKeyPackage::deserialize(data).map_err(|_| AppSW::InvalidPublicPackage)?;

        Ok(parsed)
    }

    #[inline(never)]
    pub fn load_key_package(data: &[u8]) -> Result<KeyPackageGuard, AppSW> {
        zlog_stack("start load_key_package\0");

        let status = DkgKeysReader::get_keys_status(data)?;
        match status {
            DkgKeyStatus::Completed => {}
            _ => {
                return Err(AppSW::InvalidDkgStatus);
            }
        }

        let mut start = DkgKeysReader::get_u16(data, KEY_PACKAGE_POS);
        let len = DkgKeysReader::get_u16(data, start);
        start += 2;

        let data = DkgKeysReader::get_slice(data, start, start + len);

        let package = KeyPackageGuard::deserialize(data)?;

        Ok(package)
    }

    #[inline(never)]
    pub fn load_min_signers(data: &[u8]) -> Result<usize, AppSW> {
        zlog_stack("start load_min_signers\0");

        let status = DkgKeysReader::get_keys_status(data)?;
        match status {
            DkgKeyStatus::Completed => {}
            _ => {
                return Err(AppSW::InvalidDkgStatus);
            }
        }

        let min_signers = DkgKeysReader::get_element(data, MIN_SIGNERS_POS);
        Ok(min_signers as usize)
    }

    #[inline(never)]
    pub fn load_identity_index(data: &[u8]) -> Result<usize, AppSW> {
        zlog_stack("start load_identity_index\0");

        let status = DkgKeysReader::get_keys_status(data)?;
        match status {
            DkgKeyStatus::Completed => {}
            _ => {
                return Err(AppSW::InvalidDkgStatus);
            }
        }

        let identity_index = DkgKeysReader::get_element(data, IDENTITY_INDEX_POS);
        Ok(identity_index as usize)
    }

    #[inline(never)]
    pub fn load_identities(data: &[u8]) -> Result<Vec<Identity>, AppSW> {
        zlog_stack("start load_identities\0");

        let status = DkgKeysReader::get_keys_status(data)?;
        match status {
            DkgKeyStatus::Completed => {}
            _ => {
                return Err(AppSW::InvalidDkgStatus);
            }
        }

        let mut start = DkgKeysReader::get_u16(data, IDENTITIES_POS);
        let len = DkgKeysReader::get_u16(data, start);
        start += 2;

        let end = start + len;
        let mut identities: Vec<Identity> = Vec::new();
        while start < end {
            let data = DkgKeysReader::get_slice(data, start, start + IDENTITY_LEN);
            let identity = Identity::deserialize_from(data).map_err(|_| AppSW::InvalidIdentity)?;
            start += IDENTITY_LEN;

            identities.push(identity);
        }

        if start != end {
            return Err(AppSW::InvalidPayload);
        }

        Ok(identities)
    }

    #[inline(never)]
    pub fn backup_keys(data: &[u8]) -> Result<Vec<u8>, AppSW> {
        zlog_stack("start backup_keys\0");

        let status = DkgKeysReader::get_keys_status(data)?;
        match status {
            DkgKeyStatus::Completed => {}
            _ => {
                return Err(AppSW::InvalidDkgStatus);
            }
        }

        let mut pos = DkgKeysReader::get_u16(data, FROST_PUBLIC_PACKAGE_POS);
        let len = DkgKeysReader::get_u16(data, pos);
        pos += 2 + len;

        let data = DkgKeysReader::get_slice(data, 0, pos);
        Ok(data.to_vec())
    }
}
