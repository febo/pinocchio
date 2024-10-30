use core::mem::MaybeUninit;

use pinocchio::{
    account_info::AccountInfo, instruction::{AccountMeta, Instruction, Signer}, program::invoke_signed, ProgramResult
};

/// Mints new tokens to an account.
///
/// ### Accounts:
///   0. `[WRITE]` The mint.
///   1. `[WRITE]` The account to mint tokens to.
///   2. `[SIGNER]` The mint's minting authority.
/// 
pub struct MintTo<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Token Account.
    pub token: &'a AccountInfo,
    /// Mint Authority
    pub mint_authority: &'a AccountInfo,
    /// Amount
    pub amount:  u64,
}

impl<'a> MintTo<'a> {
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
        let mut instruction_data = MaybeUninit::<[u8; 9]>::uninit();

        // Populate data
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            // Set discriminator as u8 at offset [0]
            *ptr = 7;
            // Set amount as u64 at offset [1]
            *(ptr.add(1) as *mut u64) = self.amount;
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { &instruction_data.assume_init() },
        };

        invoke_signed(
            &instruction, 
            &[self.mint, self.token, self.mint_authority], 
            signers
        )
    }
}