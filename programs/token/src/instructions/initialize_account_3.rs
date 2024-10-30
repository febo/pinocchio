use core::mem::MaybeUninit;

use pinocchio::{
    account_info::AccountInfo, instruction::{AccountMeta, Instruction, Signer}, program::invoke_signed, pubkey::Pubkey, ProgramResult
};

/// Initialize a new Token Account.
///
/// ### Accounts:
///   0. `[WRITE]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
pub struct InitilizeAccount3<'a> {
    /// New Account.
    pub token: &'a AccountInfo,
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Owner of the new Account.
    pub owner: Pubkey
}

impl<'a> InitilizeAccount3<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.token.key()),
            AccountMeta::readonly(self.mint.key()),
        ];

        // instruction data
        // -  [0]: instruction discriminator
        // -  [1..33]: owner
        let mut instruction_data = MaybeUninit::<[u8; 33]>::uninit();

        // Populate data
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            // Set discriminator as u8 at offset [0]
            *ptr = 18;
            // Set owner as Pubkey at offset [1..33]
            *(ptr.add(1) as *mut Pubkey) = self.owner;
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { &instruction_data.assume_init() },
        };

        invoke_signed(
            &instruction, 
            &[self.token, self.mint], 
            signers
        )
    }
}