use core::mem::MaybeUninit;

use pinocchio::{
    account_info::AccountInfo, instruction::{AccountMeta, Instruction, Signer}, program::invoke_signed, pubkey::Pubkey, ProgramResult
};

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
    authority_type: AuthorityType,

    /// The new authority
    new_authority: Option<Pubkey>,
}

impl<'a> SetAuthority<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable_signer(self.account.key()),
            AccountMeta::readonly_signer(self.authority.key())
        ];

        // instruction data
        // -  [0..4]: instruction discriminator
        // -  [4]: authority_type
        // -  [5..38] new_authority
        let mut instruction_data = MaybeUninit::<[u8; 12]>::uninit();

        // data
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;

            *(ptr as *mut u32) = 6;

            *(ptr.add(4) as *mut AuthorityType) = self.authority_type;

            if self.new_authority.is_some() {
                *(ptr.add(5) as *mut  u32) = 1;
                *(ptr.add(9) as *mut Pubkey) = self.new_authority.unwrap_unchecked();
            } else { 
                *(ptr.add(5) as *mut [u8; 36]) = [0; 36];
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