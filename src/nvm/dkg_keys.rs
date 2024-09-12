use ironfish_frost::frost::keys::KeyPackage;
use ledger_device_sdk::nvm::*;
use ledger_device_sdk::NVMData;
use ironfish_frost::frost::keys::PublicKeyPackage as FrostPublicKeyPackage;
use ironfish_frost::dkg::group_key::{GroupSecretKey, GROUP_SECRET_KEY_LEN};
use ironfish_frost::participant::{Identity, IDENTITY_LEN};
use alloc::vec::Vec;
use crate::AppSW;
use crate::utils::{zlog_stack};

// This is necessary to store the object in NVM and not in RAM
pub const DKG_KEYS_MAX_SIZE: usize = 3000;
const IDENTITIES_POS: usize = 0;
const MIN_SIGNERS_POS: usize = 2;
const KEY_PACKAGE_POS: usize = 4;
const GROUP_KEY_PACKAGE_POS: usize = 6;
const FROST_PUBLIC_PACKAGE_POS: usize = 8;
const DATA_STARTING_POS: u16 = 10;

// 2 bytes identitiy position
#[link_section = ".nvm_data"]
static mut DATA: NVMData<AlignedStorage<[u8; DKG_KEYS_MAX_SIZE]>> =
    NVMData::new(AlignedStorage::new([0u8; DKG_KEYS_MAX_SIZE]));

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
    pub fn get_mut_ref(&mut self) -> &mut AlignedStorage<[u8; DKG_KEYS_MAX_SIZE]> {
        unsafe { DATA.get_mut() }
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn get_element(&self, index: usize) -> u8 {
        let buffer = unsafe { DATA.get_mut() };
        buffer.get_ref()[index]
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn set_element(&self, index: usize, value: u8) {
        let mut updated_data: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };
        updated_data[index] = value;
        unsafe {
            DATA.get_mut().update(&updated_data);
        }
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn set_slice(&self, mut index: usize, value: &[u8]) {
        let mut updated_data: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };
        for b in value.iter() {
            updated_data[index] = *b;
            index += 1;
        }
        unsafe {
            DATA.get_mut().update(&updated_data);
        }
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn set_slice_with_len(&self, mut index: usize, value: &[u8]) -> usize {
        let mut updated_data: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };
        let len = value.len();
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

        index
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn get_slice(&self, start_pos: usize, end_pos:usize) -> &[u8] {
        let buffer = unsafe { DATA.get_mut() };
        &buffer.get_ref()[start_pos..end_pos]
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn set_u16(&self, mut index: usize, value: u16) -> usize{
        let mut updated_data: [u8; DKG_KEYS_MAX_SIZE] = unsafe { *DATA.get_mut().get_ref() };
        updated_data[index] = (value >> 8) as u8;
        index += 1;
        updated_data[index] = (value & 0xff) as u8;
        index += 1;
        unsafe {
            DATA.get_mut().update(&updated_data);
        }
        index
    }

    #[inline(never)]
    #[allow(unused)]
    pub fn get_u16(&self, start_pos: usize) -> usize {
        let buffer = unsafe { DATA.get_mut() };
        let buffer_ref = buffer.get_ref();
        ((buffer_ref[start_pos] as u16) << 8 | buffer_ref[start_pos+1] as u16) as usize
    }

    #[inline(never)]
    pub fn save_round_1_data(&self, identities: &Vec<Identity>, min_signers:u8) -> Result<(), AppSW>{
        self.set_u16(0, DATA_STARTING_POS);

        let mut pos = DATA_STARTING_POS as usize;
        self.set_u16(pos, (identities.len() * IDENTITY_LEN) as u16);
        pos += 2;

        for i in identities.into_iter(){
            let slice = i.serialize();
            self.set_slice(pos, slice.as_slice());
            pos += IDENTITY_LEN;
        }

        self.set_u16(MIN_SIGNERS_POS, pos as u16);
        self.set_u16(pos, min_signers as u16);

        Ok(())
    }

    #[inline(never)]
    pub fn save_keys(&self, key_package: KeyPackage, public_key_package: FrostPublicKeyPackage, group_secret_key: GroupSecretKey) {
        // Read where the previous data end up
        let mut start: usize = self.get_u16(MIN_SIGNERS_POS);
        start += 2;

        self.set_u16(KEY_PACKAGE_POS, start as u16);
        let mut pos = self.set_slice_with_len(start, key_package.serialize().unwrap().as_slice());
        self.set_u16(GROUP_KEY_PACKAGE_POS, pos as u16);
        pos = self.set_slice_with_len(pos, group_secret_key.as_slice());
        self.set_u16(FROST_PUBLIC_PACKAGE_POS, pos as u16);
        self.set_slice_with_len(pos, public_key_package.serialize().unwrap().as_slice());

        // TODO check that last pos is not bigger than dkg_keys buffer
    }

    #[inline(never)]
    pub fn load_group_secret_key(&self) -> Result<GroupSecretKey, AppSW>{
        zlog_stack("start load_group_secret_key\0");

        let mut start = self.get_u16(GROUP_KEY_PACKAGE_POS);
        let len = self.get_u16(start);
        start += 2;

        let raw = self.get_slice(start, start+len);
        let parsed = <&[u8; GROUP_SECRET_KEY_LEN]>::try_from(raw).map_err(|_| AppSW::InvalidGroupSecretKey)?;

        Ok(*parsed)
    }

    #[inline(never)]
    pub fn load_frost_public_key_package(&self) -> Result<FrostPublicKeyPackage, AppSW>{
        zlog_stack("start load_frost_public_key_package\0");

        let mut start = self.get_u16(FROST_PUBLIC_PACKAGE_POS);
        let len = self.get_u16(start);
        start += 2;

        let data = self.get_slice(start, start+len);
        let parsed = FrostPublicKeyPackage::deserialize(data).map_err(|_| AppSW::InvalidPublicPackage)?;

        Ok(parsed)
    }


    #[inline(never)]
    pub fn load_key_package(&self) -> Result<KeyPackage, AppSW>{
        zlog_stack("start load_key_package\0");

        let mut start = self.get_u16(KEY_PACKAGE_POS);
        let len = self.get_u16(start);
        start += 2;

        let data = self.get_slice(start, start+len);
        let package = KeyPackage::deserialize(data).map_err(|_| AppSW::InvalidKeyPackage)?;

        Ok(package)
    }

    #[inline(never)]
    pub fn load_min_signers(&self) -> Result<usize, AppSW>{
        zlog_stack("start load_min_signers\0");

        let start = self.get_u16(MIN_SIGNERS_POS);
        Ok(self.get_u16(start))
    }

    #[inline(never)]
    pub fn load_identities(&self) -> Result<Vec<Identity>, AppSW>{
        zlog_stack("start load_identities\0");

        let mut start = self.get_u16(IDENTITIES_POS);
        let len = self.get_u16(start);
        start += 2;

        let end = start + len;
        let mut identities:Vec<Identity> = Vec::new();
        while start < end {
            let data = self.get_slice(start,start+IDENTITY_LEN);
            let identity
                = Identity::deserialize_from(data).map_err(|_| AppSW::InvalidIdentity)?;
            start += IDENTITY_LEN;

            identities.push(identity);
        }

        if start != end {
            return Err(AppSW::InvalidPayload);
        }

        Ok(identities)
    }
}
