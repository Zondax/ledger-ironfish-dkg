/*****************************************************************************
 *   Ledger App Ironfish Rust.
 *   (c) 2023 Ledger SAS.
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 *****************************************************************************/
use crate::AppSW;
use core::str::FromStr;
use ledger_device_sdk::io;

#[inline(never)]
pub fn handler_get_version(comm: &mut io::Comm) -> Result<(), AppSW> {
    let v = option_env!("APPVERSION_STR").unwrap_or("v0.0.0");
    if let Some((major, minor, patch)) = parse_version_string(v) {
        let mut resp: [u8; 8] = [0u8; 8];

        // APP TESTING
        resp[0..1].copy_from_slice(&[0u8]);

        // APP VERSION
        resp[1..3].copy_from_slice(major.to_be_bytes().as_slice());
        resp[3..5].copy_from_slice(minor.to_be_bytes().as_slice());
        resp[5..7].copy_from_slice(patch.to_be_bytes().as_slice());

        // !IS_UX_ALLOWED
        resp[7..8].copy_from_slice(&[0u8]);

        comm.append(&resp);
        Ok(())
    } else {
        Err(AppSW::VersionParsingFail)
    }
}

fn parse_version_string(input: &str) -> Option<(u16, u16, u16)> {
    // Split the input string by '.'.
    // Input should be of the form "major.minor.patch",
    // where "major", "minor", and "patch" are integers.

    let mut parts = input[1..].split('.');
    let major = u16::from_str(parts.next()?).ok()?;
    let minor = u16::from_str(parts.next()?).ok()?;
    let patch = u16::from_str(parts.next()?).ok()?;
    Some((major, minor, patch))
}
