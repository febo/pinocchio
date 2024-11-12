use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{IxData, UNINIT_BYTE};

/// Initialize a new Token Account.
///
/// ### Accounts:
///   0. `[WRITE]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
///   3. `[]` Rent sysvar
pub struct InitilizeAccount2<'a> {
    /// New Account.
    pub token: &'a AccountInfo,
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Rent Sysvar Account
    pub rent_sysvar: &'a AccountInfo,
    /// Owner of the new Account.
    pub owner: &'a Pubkey,
}

impl<'a> InitilizeAccount2<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.token.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly(self.rent_sysvar.key()),
        ];

        // instruction data
        // -  [0]: instruction discriminator
        // -  [1..33]: owner

        let mut ix_buffer = [UNINIT_BYTE; 33];
        let mut ix_data = IxData::new(&mut ix_buffer);

        // Set discriminator as u8 at offset [0]
        ix_data.write_bytes(&[16]);

        // Set owner as [u8; 32] at offset [1..33]
        ix_data.write_bytes(self.owner.as_ref());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: ix_data.read_bytes(),
        };

        invoke_signed(
            &instruction,
            &[self.token, self.mint, self.rent_sysvar],
            signers,
        )
    }
}
