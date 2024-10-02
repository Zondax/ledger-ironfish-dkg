/////////
/////////
/////////

use crate::AppSW;
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};
use core::ptr;
use ironfish_frost::dkg::group_key::{GroupSecretKey, GROUP_SECRET_KEY_LEN};
use ironfish_frost::frost::keys::KeyPackage;
use ironfish_frost::participant::Secret as IronfishSecret;

const SECRET_KEY_LEN: usize = 32;

pub struct KeyPackageGuard {
    secret: KeyPackage,
}

impl KeyPackageGuard {
    pub fn new(secret: KeyPackage) -> Self {
        KeyPackageGuard { secret }
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, AppSW> {
        let secret = KeyPackage::deserialize(data).map_err(|_| AppSW::InvalidKeyPackage)?;
        Ok(KeyPackageGuard { secret })
    }
}

impl Drop for KeyPackageGuard {
    fn drop(&mut self) {
        unsafe {
            ptr::write_bytes(&mut self.secret as *mut KeyPackage, 0, 1);
        }
    }
}

impl Deref for KeyPackageGuard {
    type Target = KeyPackage;

    fn deref(&self) -> &Self::Target {
        &self.secret
    }
}

impl DerefMut for KeyPackageGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.secret
    }
}

/////////
/////////
/////////

pub struct IronfishSecretGuard {
    secret: IronfishSecret,
}

impl IronfishSecretGuard {
    pub fn from_secret_keys(secret_key_0: &[u8], secret_key_1: &[u8]) -> Self {
        let secret = IronfishSecret::from_secret_keys(
            secret_key_0[0..SECRET_KEY_LEN].try_into().unwrap(),
            secret_key_1[0..SECRET_KEY_LEN].try_into().unwrap(),
        );
        IronfishSecretGuard { secret }
    }
}

impl Drop for IronfishSecretGuard {
    fn drop(&mut self) {
        unsafe {
            ptr::write_bytes(&mut self.secret as *mut IronfishSecret, 0, 1);
        }
    }
}

impl Deref for IronfishSecretGuard {
    type Target = IronfishSecret;

    fn deref(&self) -> &Self::Target {
        &self.secret
    }
}

impl DerefMut for IronfishSecretGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.secret
    }
}

/////////
/////////
/////////

pub struct GroupSecretKeyGuard {
    secret: GroupSecretKey,
}

impl GroupSecretKeyGuard {
    pub fn from_raw(data: &[u8]) -> Result<Self, AppSW> {
        let secret = <&[u8; GROUP_SECRET_KEY_LEN]>::try_from(data)
            .map_err(|_| AppSW::InvalidGroupSecretKey)?;

        Ok(GroupSecretKeyGuard { secret: *secret })
    }
}

impl Drop for GroupSecretKeyGuard {
    fn drop(&mut self) {
        unsafe {
            ptr::write_bytes(self.secret.as_mut_ptr(), 0, GROUP_SECRET_KEY_LEN);
        }
    }
}

impl Deref for GroupSecretKeyGuard {
    type Target = GroupSecretKey;

    fn deref(&self) -> &Self::Target {
        &self.secret
    }
}

impl DerefMut for GroupSecretKeyGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.secret
    }
}

/////////
/////////
/////////

pub struct EncryptionKeyGuard {
    secret: [u8; SECRET_KEY_LEN],
}

impl EncryptionKeyGuard {
    pub fn from_secret_keys(secret_key_0: &[u8]) -> Self {
        let secret = secret_key_0[0..SECRET_KEY_LEN].try_into().unwrap();

        EncryptionKeyGuard { secret }
    }
}

impl Drop for EncryptionKeyGuard {
    fn drop(&mut self) {
        unsafe {
            ptr::write_bytes(self.secret.as_mut_ptr(), 0, self.secret.len());
        }
    }
}

impl Deref for EncryptionKeyGuard {
    type Target = [u8; SECRET_KEY_LEN];

    fn deref(&self) -> &Self::Target {
        &self.secret
    }
}

impl DerefMut for EncryptionKeyGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.secret
    }
}

/////////
/////////
/////////

pub struct KeysDataGuard {
    secret: Vec<u8>,
}

impl KeysDataGuard {
    pub fn new(secret: Vec<u8>) -> Self {
        KeysDataGuard { secret }
    }
}

impl Drop for KeysDataGuard {
    fn drop(&mut self) {
        unsafe {
            ptr::write_bytes(self.secret.as_mut_ptr(), 0, self.secret.len());
        }
    }
}

impl Deref for KeysDataGuard {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.secret.as_slice()
    }
}

impl DerefMut for KeysDataGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.secret.as_mut_slice()
    }
}
