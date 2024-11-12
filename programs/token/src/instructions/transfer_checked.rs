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
///   0. `[WRITE]` The source account.
///   1. `[]` The token mint.
///   2. `[WRITE]` The destination account.
///   3. `[SIGNER]` The source account's owner/delegate.
pub struct TransferChecked<'a> {
    /// Sender account.
    pub from: &'a AccountInfo,
    /// Mint Account
    pub mint: &'a AccountInfo,
    /// Recipient account.
    pub to: &'a AccountInfo,
    /// Authority account.
    pub authority: &'a AccountInfo,
    /// Amount of microtokens to transfer.
    pub amount: u64,
    /// Decimal for the Token
    pub decimals: u8,
}

impl<'a> TransferChecked<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.from.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::writable(self.to.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..9]: amount (8 bytes, u64)
        // -  [9]: decimals (1 byte, u8)
        let mut instruction_data = [UNINIT_BYTE; 10];

        // Set discriminator as u8 at offset [0]
        ix_data.write_bytes(&[12]);
        // Set amount as u64 at offset [1..9]
        ix_data.write_bytes(&self.amount.to_le_bytes());
        // Set decimals as u8 at offset [9]
        ix_data.write_bytes(&[self.decimals]);

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: ix_data.read_bytes(),
        };

        invoke_signed(&instruction, &[self.from, self.to, self.authority], signers)
    }
}
