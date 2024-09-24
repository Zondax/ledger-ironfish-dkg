use crate::AppSW;
#[cfg(feature = "ledger")]
use ledger_device_sdk::io::ApduHeader;

/// Possible input commands received through APDUs.
pub enum Instruction {
    GetVersion,
    DkgGetIdentity { review: bool },
    DkgGetPublicPackage,
    DkgRound1 { chunk: u8 },
    DkgRound2 { chunk: u8 },
    DkgRound3Min { chunk: u8 },
    DkgCommitments { chunk: u8 },
    DkgSign { chunk: u8 },
    DkgGetKeys { key_type: u8 },
    DkgBackupKeys,
    DkgRestoreKeys { chunk: u8 },
    GetResult { chunk: u8 },
    ReviewTx { chunk: u8 },
}

#[cfg(feature = "ledger")]
impl TryFrom<ApduHeader> for Instruction {
    type Error = AppSW;

    /// APDU parsing logic.
    ///
    /// Parses INS, P1 and P2 bytes to build an [`Instruction`]. P1 and P2 are translated to
    /// strongly typed variables depending on the APDU instruction code. Invalid INS, P1 or P2
    /// values result in errors with a status word, which are automatically sent to the host by the
    /// SDK.
    ///
    /// This design allows a clear separation of the APDU parsing logic and commands handling.
    ///
    /// Note that CLA is not checked here. Instead the method [`Comm::set_expected_cla`] is used in
    /// [`sample_main`] to have this verification automatically performed by the SDK.
    fn try_from(value: ApduHeader) -> Result<Self, Self::Error> {
        match (value.ins, value.p1, value.p2) {
            (0x00, 0, 0) => Ok(Instruction::GetVersion),
            (0x11, 0..=1, 0) => Ok(Instruction::DkgGetIdentity {
                review: value.p1 == 1,
            }),
            (0x12, 0..=2, 0) => Ok(Instruction::DkgRound1 { chunk: value.p1 }),
            (0x13, 0..=2, 0) => Ok(Instruction::DkgRound2 { chunk: value.p1 }),
            (0x14, 0..=2, 0) => Ok(Instruction::DkgRound3Min { chunk: value.p1 }),
            (0x15, 0..=2, 0) => Ok(Instruction::DkgCommitments { chunk: value.p1 }),
            (0x16, 0..=2, 0) => Ok(Instruction::DkgSign { chunk: value.p1 }),
            (0x17, 0, 0..=3) => Ok(Instruction::DkgGetKeys { key_type: value.p2 }),
            (0x18, 0..=2, 0) => Ok(Instruction::DkgGetPublicPackage),
            (0x19, 0, 0) => Ok(Instruction::DkgBackupKeys),
            (0x1a, 0..=2, 0) => Ok(Instruction::DkgRestoreKeys { chunk: value.p1 }),
            (0x1b, 0..=255, 0) => Ok(Instruction::GetResult { chunk: value.p1 }),
            (0x1c, 0..=2, 0) => Ok(Instruction::ReviewTx { chunk: value.p1 }),
            // Any supported ins with wrong p1 p2 should fall here
            (0x00, _, _) => Err(AppSW::WrongP1P2),
            (0x11..=0x1c, _, _) => Err(AppSW::WrongP1P2),
            // Any other value (unsupported ins) should fall here
            (_, _, _) => Err(AppSW::InsNotSupported),
        }
    }
}
