use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{write_bytes, UNINIT_BYTE};

use super::TokenProgramVariant;

/// Initialize a new mint.
///
/// ### Accounts:
///   0. `[WRITABLE]` Mint account
///   1. `[]` Rent sysvar
pub struct InitializeMint<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Rent sysvar Account.
    pub rent_sysvar: &'a AccountInfo,
    /// Decimals.
    pub decimals: u8,
    /// Mint Authority.
    pub mint_authority: &'a Pubkey,
    /// Freeze Authority.
    pub freeze_authority: Option<&'a Pubkey>,
}

impl<'a> InitializeMint<'a> {
    #[inline(always)]
    pub fn invoke(&self, token_program: TokenProgramVariant) -> ProgramResult {
        self.invoke_signed(&[], token_program)
    }

    pub fn invoke_signed(
        &self,
        signers: &[Signer],
        token_program: TokenProgramVariant,
    ) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly(self.rent_sysvar.key()),
        ];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1]: decimals (1 byte, u8)
        // -  [2..34]: mint_authority (32 bytes, Pubkey)
        // -  [34]: freeze_authority presence flag (1 byte, u8)
        // -  [35..67]: freeze_authority (optional, 32 bytes, Pubkey)
        let mut instruction_data = [UNINIT_BYTE; 67];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[0]);
        // Set decimals as u8 at offset [1]
        write_bytes(&mut instruction_data[1..2], &[self.decimals]);
        // Set mint_authority as Pubkey at offset [2..34]
        write_bytes(&mut instruction_data[2..34], self.mint_authority);
        // Set COption & freeze_authority at offset [34..67]
        if let Some(freeze_auth) = self.freeze_authority {
            write_bytes(&mut instruction_data[34..35], &[1]);
            write_bytes(&mut instruction_data[35..], freeze_auth);
        } else {
            write_bytes(&mut instruction_data[34..35], &[0]);
        }

        let instruction = Instruction {
            program_id: &token_program.into(),
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 67) },
        };

        invoke_signed(&instruction, &[self.mint, self.rent_sysvar], signers)
    }
}
