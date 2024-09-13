use crate::bolos::zlog_stack;
use crate::AppSW;
use alloc::vec::Vec;
use ironfish_frost::dkg::group_key::{GroupSecretKey, GROUP_SECRET_KEY_LEN};
use ironfish_frost::frost::keys::KeyPackage;
use ironfish_frost::frost::keys::PublicKeyPackage as FrostPublicKeyPackage;
use ironfish_frost::participant::{Identity, IDENTITY_LEN};
use ledger_device_sdk::nvm::*;
use ledger_device_sdk::NVMData;

// This is necessary to store the object in NVM and not in RAM
pub const DKG_KEYS_MAX_SIZE: usize = 3000;

const DKG_STATUS: usize = 0;
const DKG_VERSION: usize = 1;
const IDENTITIES_POS: usize = 2;
const MIN_SIGNERS_POS: usize = 4;
const KEY_PACKAGE_POS: usize = 6;
const GROUP_KEY_PACKAGE_POS: usize = 8;
const FROST_PUBLIC_PACKAGE_POS: usize = 10;
const DATA_STARTING_POS: u16 = 12;

enum DkgKeyStatus {
    Idle,
    Initiated,
    Completed
}

enum DkgKeyVersion{
    V1 = 1
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
    #[inline(never)]
    #[allow(unused)]
    pub fn get_mut_ref(&mut self) -> &mut SafeStorage<[u8; DKG_KEYS_MAX_SIZE]> {
        unsafe { DATA.get_mut() }
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
    pub fn get_element(&self, index: usize) -> u8 {
        let buffer = unsafe { DATA.get_mut() };
        buffer.get_ref()[index]
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn set_element(&self, index: usize, value: u8) -> Result<(), AppSW> {
        self.check_write_pos(index)?;

        let mut updated_data: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };
        updated_data[index] = value;
        unsafe {
            DATA.get_mut().update(&updated_data);
        }

        self.is_valid_write()?;
        Ok(())
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn set_slice(&self, mut index: usize, value: &[u8]) -> Result<(), AppSW> {
        self.check_write_pos(index + value.len())?;

        let mut updated_data: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };
        for b in value.iter() {
            updated_data[index] = *b;
            index += 1;
        }
        unsafe {
            DATA.get_mut().update(&updated_data);
        }

        self.is_valid_write()?;
        Ok(())
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn set_slice_with_len(&self, mut index: usize, value: &[u8]) -> Result<usize, AppSW> {
        let len = value.len();
        self.check_write_pos(index + 2 + len)?;

        let mut updated_data: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };

        updated_data[index] = (len >> 8) as u8;
        index += 1;

        updated_data[index] = (len & 0xff) as u8;
        index += 1;

        for b in value.iter() {
            updated_data[index] = *b;
            index += 1;
        }
        unsafe {
            DATA.get_mut().update(&updated_data);
        }

        self.is_valid_write()?;
        Ok(index)
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn get_slice(&self, start_pos: usize, end_pos: usize) -> &[u8] {
        let buffer = unsafe { DATA.get_mut() };
        &buffer.get_ref()[start_pos..end_pos]
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn set_u16(&self, mut index: usize, value: u16) -> Result<usize, AppSW> {
        self.check_write_pos(index + 2)?;

        let mut updated_data: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };
        updated_data[index] = (value >> 8) as u8;
        index += 1;
        updated_data[index] = (value & 0xff) as u8;
        index += 1;
        unsafe {
            DATA.get_mut().update(&updated_data);
        }

        self.is_valid_write()?;
        Ok(index)
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn get_u16(&self, start_pos: usize) -> usize {
        let buffer = unsafe { DATA.get_mut() };
        let buffer_ref = buffer.get_ref();
        ((buffer_ref[start_pos] as u16) << 8 | buffer_ref[start_pos + 1] as u16) as usize
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
    ) -> Result<(), AppSW> {
        zlog_stack("start save_round_1_data\0");

        self.set_u16(IDENTITIES_POS, DATA_STARTING_POS)?;

        let mut pos = DATA_STARTING_POS as usize;
        self.set_u16(pos, (identities.len() * IDENTITY_LEN) as u16)?;
        pos += 2;

        for i in identities.into_iter() {
            let slice = i.serialize();
            self.set_slice(pos, slice.as_slice())?;
            pos += IDENTITY_LEN;
        }

        self.set_u16(MIN_SIGNERS_POS, pos as u16)?;
        self.set_u16(pos, min_signers as u16)?;

        self.update_keys_status(DkgKeyStatus::Initiated, DkgKeyVersion::V1)
    }


    #[inline(never)]
    pub fn update_keys_status(
        &self,
        status: DkgKeyStatus,
        version: DkgKeyVersion
    ) -> Result<(), AppSW> {
        zlog_stack("start update_keys_status\0");

        match version {
           DkgKeyVersion::V1 => self.set_element(DKG_VERSION, 1)
        }?;

        match status {
            DkgKeyStatus::Idle => self.set_element(DKG_STATUS, 0),
            DkgKeyStatus::Initiated => self.set_element(DKG_STATUS, 1),
            DkgKeyStatus::Completed => self.set_element(DKG_STATUS, 2)
        }
    }

    #[inline(never)]
    pub fn get_keys_status(&self
    ) -> Result<DkgKeyStatus, AppSW> {
        zlog_stack("start get_keys_status\0");

        let status = self.get_element(DKG_STATUS);
        match status {
            0 => Ok(DkgKeyStatus::Idle),
            1 => Ok(DkgKeyStatus::Initiated),
            2 => Ok(DkgKeyStatus::Completed),
            _ => Err(AppSW::InvalidDkgStatus)
        }
    }

    #[inline(never)]
    pub fn save_keys(
        &self,
        key_package: KeyPackage,
        public_key_package: FrostPublicKeyPackage,
        group_secret_key: GroupSecretKey,
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
        let mut start: usize = self.get_u16(MIN_SIGNERS_POS);
        start += 2;

        self.set_u16(KEY_PACKAGE_POS, start as u16)?;
        let mut pos =
            self.set_slice_with_len(start, key_package.serialize().unwrap().as_slice())?;
        self.set_u16(GROUP_KEY_PACKAGE_POS, pos as u16)?;
        pos = self.set_slice_with_len(pos, group_secret_key.as_slice())?;
        self.set_u16(FROST_PUBLIC_PACKAGE_POS, pos as u16)?;
        self.set_slice_with_len(pos, public_key_package.serialize().unwrap().as_slice())?;

        self.update_keys_status(DkgKeyStatus::Completed, DkgKeyVersion::V1)
    }

    #[inline(never)]
    pub fn load_group_secret_key(&self) -> Result<GroupSecretKey, AppSW> {
        zlog_stack("start load_group_secret_key\0");

        let status = self.get_keys_status()?;
        match status {
            DkgKeyStatus::Completed => {}
            _ => {
                return Err(AppSW::InvalidDkgStatus);
            }
        }

        let mut start = self.get_u16(GROUP_KEY_PACKAGE_POS);
        let len = self.get_u16(start);
        start += 2;

        let raw = self.get_slice(start, start + len);
        let parsed = <&[u8; GROUP_SECRET_KEY_LEN]>::try_from(raw)
            .map_err(|_| AppSW::InvalidGroupSecretKey)?;

        Ok(*parsed)
    }

    #[inline(never)]
    pub fn load_frost_public_key_package(&self) -> Result<FrostPublicKeyPackage, AppSW> {
        zlog_stack("start load_frost_public_key_package\0");

        let status = self.get_keys_status()?;
        match status {
            DkgKeyStatus::Completed => {}
            _ => {
                return Err(AppSW::InvalidDkgStatus);
            }
        }

        let mut start = self.get_u16(FROST_PUBLIC_PACKAGE_POS);
        let len = self.get_u16(start);
        start += 2;

        let data = self.get_slice(start, start + len);
        let parsed =
            FrostPublicKeyPackage::deserialize(data).map_err(|_| AppSW::InvalidPublicPackage)?;

        Ok(parsed)
    }

    #[inline(never)]
    pub fn load_key_package(&self) -> Result<KeyPackage, AppSW> {
        zlog_stack("start load_key_package\0");

        let status = self.get_keys_status()?;
        match status {
            DkgKeyStatus::Completed => {}
            _ => {
                return Err(AppSW::InvalidDkgStatus);
            }
        }

        let mut start = self.get_u16(KEY_PACKAGE_POS);
        let len = self.get_u16(start);
        start += 2;

        let data = self.get_slice(start, start + len);
        let package = KeyPackage::deserialize(data).map_err(|_| AppSW::InvalidKeyPackage)?;

        Ok(package)
    }

    #[inline(never)]
    pub fn load_min_signers(&self) -> Result<usize, AppSW> {
        zlog_stack("start load_min_signers\0");

        let status = self.get_keys_status()?;
        match status {
            DkgKeyStatus::Completed => {}
            _ => {
                return Err(AppSW::InvalidDkgStatus);
            }
        }

        let start = self.get_u16(MIN_SIGNERS_POS);
        Ok(self.get_u16(start))
    }

    #[inline(never)]
    pub fn load_identities(&self) -> Result<Vec<Identity>, AppSW> {
        zlog_stack("start load_identities\0");

        let status = self.get_keys_status()?;
        match status {
            DkgKeyStatus::Completed => {}
            _ => {
                return Err(AppSW::InvalidDkgStatus);
            }
        }

        let mut start = self.get_u16(IDENTITIES_POS);
        let len = self.get_u16(start);
        start += 2;

        let end = start + len;
        let mut identities: Vec<Identity> = Vec::new();
        while start < end {
            let data = self.get_slice(start, start + IDENTITY_LEN);
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
    pub fn backup_keys(&self) -> Result<&[u8], AppSW> {
        zlog_stack("start backup_keys\0");

        let status = self.get_keys_status()?;
        match status {
            DkgKeyStatus::Completed => {}
            _ => {
                return Err(AppSW::InvalidDkgStatus);
            }
        }

        let mut pos = self.get_u16(FROST_PUBLIC_PACKAGE_POS);
        let len = self.get_u16(pos);
        pos += 2 + len;

        let data = self.get_slice(0, pos);
        Ok(data)
    }

    #[inline(never)]
    pub fn restore_keys(&self, data: &[u8]) -> Result<(), AppSW> {
        zlog_stack("start restore_keys\0");

        if data[1] != 1{
            return Err(AppSW::InvalidDkgKeysVersion);
        }

        self.set_slice(0, data)
    }
}
