/////////
/////////
/////////

use crate::AppSW;
use core::ops::{Deref, DerefMut};
use core::ptr;
use ironfish_frost::frost::keys::KeyPackage;

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
