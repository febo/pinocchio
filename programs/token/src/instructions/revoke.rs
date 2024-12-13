use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

use super::TokenProgramVariant;

/// Revokes the delegate's authority.
///
/// ### Accounts:
///   0. `[WRITE]` The source account.
///   1. `[SIGNER]` The source account owner.
pub struct Revoke<'a> {
    /// New Account.
    pub token: &'a AccountInfo,
    /// Mint Account.
    pub authority: &'a AccountInfo,
}

impl<'a> Revoke<'a> {
    #[inline(always)]
    pub fn invoke(&self, token_program: TokenProgramVariant) -> ProgramResult {
        self.invoke_signed(&[], token_program)
    }

    pub fn invoke_signed(
        &self,
        signers: &[Signer],
        token_program: TokenProgramVariant,
    ) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.token.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        let instruction = Instruction {
            program_id: &token_program.into(),
            accounts: &account_metas,
            data: &[5],
        };

        invoke_signed(&instruction, &[self.token, self.authority], signers)
    }
}
