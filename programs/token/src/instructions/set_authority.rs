use core::mem::MaybeUninit;

use pinocchio::{
    account_info::AccountInfo, instruction::{AccountMeta, Instruction, Signer}, program::invoke_signed, ProgramResult
};

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum AuthorityType {
    MintTokens,
    FreezeAccount,
    AccountOwner,
    CloseAccount,
}


/// Sets a new authority of a mint or account.
///
/// ### Accounts:
///   0. `[WRITE]` The mint or account to change the authority of.
///   1. `[SIGNER]` The current authority of the mint or account.
pub struct SetAuthority<'a> {
    /// Account (Mint or Token)
    pub account: &'a AccountInfo,

    /// Authority of the Account.
    pub authority: &'a AccountInfo,

    /// The type of authority to update.
    pub authority_type: AuthorityType,

    /// The new authority
    pub new_authority: Option<[u8;32]>,
}

impl<'a> SetAuthority<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly_signer(self.authority.key())
        ];

        // instruction data
        // -  [0]: instruction discriminator
        // -  [1]: authority_type
        // -  [2..35] new_authority
        let mut instruction_data = MaybeUninit::<[u8; 35]>::uninit();

        // data
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;

            *ptr = 6;

            *(ptr.add(1) as *mut AuthorityType) = self.authority_type;

            if self.new_authority.is_some() {
                *ptr.add(2) = 1;
                *(ptr.add(3) as *mut [u8; 32]) = self.new_authority.unwrap_unchecked();
            } else { 
                *(ptr.add(5) as *mut [u8; 33]) = [0; 33];
            }
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { &instruction_data.assume_init() },
        };

        invoke_signed(
            &instruction, 
            &[self.account, self.authority], 
            signers)
    }
}