use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

use crate::{IxData, UNINIT_BYTE};

/// Approves a delegate.
///
/// ### Accounts:
///   0. `[WRITE]` The source account.
///   1. `[]` The token mint.
///   2. `[]` The delegate.
///   3. `[SIGNER]` The source account owner.
pub struct ApproveChecked<'a> {
    /// Source Account.
    pub token: &'a AccountInfo,
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Delegate Account.
    pub delegate: &'a AccountInfo,
    /// Source Owner Account.
    pub authority: &'a AccountInfo,
    /// Amount.
    pub amount: u64,
    /// Decimals.
    pub decimals: u8,
}

impl<'a> ApproveChecked<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.token.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly(self.delegate.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data
        // -  [0]  : instruction discriminator (1 byte, u8)
        // -  [1..9]: amount (8 bytes, u64)
        // -  [9]   : decimals (1 byte, u8)
        let mut ix_buffer = [UNINIT_BYTE; 10];
        let mut ix_data = IxData::new(&mut ix_buffer);

        // Set discriminator as u8 at offset [0]
        ix_data.write_bytes(&[13]);
        // Set amount as u64 at offset [1..9]
        ix_data.write_bytes(&self.amount.to_le_bytes());
        // Set decimals as u8 at offset [9]
        ix_data.write_bytes(&[self.decimals]);

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: ix_data.read_bytes(),
        };

        invoke_signed(
            &instruction,
            &[self.token, self.mint, self.delegate, self.authority],
            signers,
        )
    }
}
