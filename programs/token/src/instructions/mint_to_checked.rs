use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

use crate::{IxData, UNINIT_BYTE};

/// Mints new tokens to an account.
///
/// ### Accounts:
///   0. `[WRITE]` The mint.
///   1. `[WRITE]` The account to mint tokens to.
///   2. `[SIGNER]` The mint's minting authority.
///
pub struct MintToChecked<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Token Account.
    pub token: &'a AccountInfo,
    /// Mint Authority
    pub mint_authority: &'a AccountInfo,
    /// Amount
    pub amount: u64,
    /// Decimals
    pub decimals: u8,
}

impl<'a> MintToChecked<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::writable(self.token.key()),
            AccountMeta::readonly_signer(self.mint_authority.key()),
        ];

        // Instruction data layout:
        // -  [0]: instruction discriminator
        // -  [1..9]: amount
        // -  [9]: decimals
        let mut ix_buffer = [UNINIT_BYTE; 10];
        let mut ix_data = IxData::new(&mut ix_buffer);

        // Set discriminator as u8 at offset [0]
        ix_data.write_bytes(&[14]);
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
            &[self.mint, self.token, self.mint_authority],
            signers,
        )
    }
}
