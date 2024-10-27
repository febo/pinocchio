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
pub struct MintToChecked<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,

    /// Token Account.
    pub token: &'a AccountInfo,

    /// Mint Authority
    pub mint_authority: &'a AccountInfo,

    /// Amount
    pub amount:  u64,

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

        // instruction data
        // -  [0..4]: instruction discriminator
        // -  [4..12]: amount
        // -  [12..13]: decimals
        let mut instruction_data = MaybeUninit::<[u8; 12]>::uninit();

        // data
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;

            *(ptr as *mut u32) = 14;

            *(ptr.add(4) as *mut u64) = self.amount;

            *(ptr.add(12) as *mut u8) = self.decimals;
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { &instruction_data.assume_init() },
        };

        invoke_signed(
            &instruction, 
            &[self.mint, self.token, self.mint_authority], 
            signers)
    }
}