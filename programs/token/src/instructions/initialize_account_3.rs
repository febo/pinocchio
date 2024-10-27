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
            AccountMeta::writable_signer(self.token.key()),
            AccountMeta::readonly(self.mint.key()),
        ];

        // instruction data
        // -  [0..4]: instruction discriminator
        // -  [4..36]: owner
        let mut instruction_data = MaybeUninit::<[u8; 12]>::uninit();

        // data
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;

            *(ptr as *mut u32) = 18;

            *(ptr.add(4) as *mut Pubkey) = self.owner;
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { &instruction_data.assume_init() },
        };

        invoke_signed(
            &instruction, 
            &[self.token, self.mint], 
            signers)
    }
}