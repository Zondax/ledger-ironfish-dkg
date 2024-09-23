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
            (0, 0, 0) => Ok(Instruction::GetVersion),
            (16, 0..=1, 0) => Ok(Instruction::DkgGetIdentity {
                review: value.p1 == 1,
            }),
            (17, 0..=2, 0) => Ok(Instruction::DkgRound1 { chunk: value.p1 }),
            (18, 0..=2, 0) => Ok(Instruction::DkgRound2 { chunk: value.p1 }),
            (19, 0..=2, 0) => Ok(Instruction::DkgRound3Min { chunk: value.p1 }),
            (20, 0..=2, 0) => Ok(Instruction::DkgCommitments { chunk: value.p1 }),
            (21, 0..=2, 0) => Ok(Instruction::DkgSign { chunk: value.p1 }),
            (22, 0, 0..=3) => Ok(Instruction::DkgGetKeys { key_type: value.p2 }),
            (24, 0..=2, 0) => Ok(Instruction::DkgGetPublicPackage),
            (25, 0, 0) => Ok(Instruction::DkgBackupKeys),
            (26, 0..=2, 0) => Ok(Instruction::DkgRestoreKeys { chunk: value.p1 }),
            (27, 0..=255, 0) => Ok(Instruction::GetResult { chunk: value.p1 }),
            (3..=4, _, _) => Err(AppSW::WrongP1P2),
            (17..=26, _, _) => Err(AppSW::WrongP1P2),
            (0x1c, 0..=2, 0) => Ok(Instruction::ReviewTx { chunk: value.p1 }),
            (_, _, _) => Err(AppSW::InsNotSupported),
        }
    }
}
