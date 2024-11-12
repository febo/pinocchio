use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

use crate::{IxData, UNINIT_BYTE};

/// Transfer Tokens from one Token Account to another.
///
/// ### Accounts:
///   0. `[WRITE]` Sender account
///   1. `[WRITE]` Recipient account
///   2. `[SIGNER]` Authority account
pub struct Transfer<'a> {
    /// Sender account.
    pub from: &'a AccountInfo,
    /// Recipient account.
    pub to: &'a AccountInfo,
    /// Authority account.
    pub authority: &'a AccountInfo,
    /// Amount of microtokens to transfer.
    pub amount: u64,
}

impl<'a> Transfer<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.from.key()),
            AccountMeta::writable(self.to.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..9]: amount (8 bytes, u64)
        let mut instruction_data = [UNINIT_BYTE; 9];

        // Set discriminator as u8 at offset [0]
        ix_data.write_bytes(&[3]);
        // Set amount as u64 at offset [1..9]
        ix_data.write_bytes(&self.amount.to_le_bytes());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: ix_data.read_bytes(),
        };

        invoke_signed(&instruction, &[self.from, self.to, self.authority], signers)
    }
}
