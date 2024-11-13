use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{FromOptPubkeyToOptBytes, IxData, UNINIT_BYTE};

/// Initialize a new mint.
///
/// ### Accounts:
///   0. `[WRITABLE]` Mint account
pub struct InitilizeMint2<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Decimals.
    pub decimals: u8,
    /// Mint Authority.
    pub mint_authority: &'a Pubkey,
    /// Freeze Authority.
    pub freeze_authority: Option<&'a Pubkey>,
}

impl<'a> InitilizeMint2<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [AccountMeta::writable(self.mint.key())];

        // Instruction data layout:
        // -  [0]: instruction discriminator
        // -  [1]: decimals
        // -  [2..34]: mint_authority
        // -  [34..35]: freeze_authority presence flag
        // -  [35..67]: freeze_authority
        let mut ix_buffer = [UNINIT_BYTE; 67];
        let mut ix_data = IxData::new(&mut ix_buffer);

        // Set discriminator as u8 at offset [0]
        ix_data.write_bytes(&[20]);
        // Set decimals as u8 at offset [1]
        ix_data.write_bytes(&[self.decimals]);
        // Set mint_authority as Pubkey at offset [2..34]
        ix_data.write_bytes(self.mint_authority.as_ref());
        // Set COption & freeze_authority at offset [34..67]
        ix_data.write_optional_bytes(self.freeze_authority.to_opt_slice());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: ix_data.read_bytes(),
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}
