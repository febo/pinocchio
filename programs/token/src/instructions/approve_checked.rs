use core::mem::MaybeUninit;

use pinocchio::{
    account_info::AccountInfo, instruction::{AccountMeta, Instruction, Signer}, program::invoke_signed, ProgramResult
};

/// Approves a delegate.
///
/// ### Accounts:
///   0. `[WRITE]` The source account.
///   1. `[]` The token mint.
///   2. `[]` The delegate.
///   3. `[SIGNER]` The source account owner.
pub struct ApproveChecked<'a> {
    /// Source Account.
    pub token: &'a AccountInfo,

    /// Mint Account.
    pub mint: &'a AccountInfo,

    /// Delegate Account
    pub delegate: &'a AccountInfo,

    /// Source Owner Account
    pub authority: &'a AccountInfo,

    /// Amount
    pub amount: u64,

    /// Decimals
    pub decimals: u8
}

impl<'a> ApproveChecked<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.token.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly(self.delegate.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // instruction data
        // -  [0..4]: instruction discriminator
        // -  [4..12]: amount
        // -  [12]: decimals

        let mut instruction_data = MaybeUninit::<[u8; 12]>::uninit();

        // data
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;

            *(ptr as *mut u32) = 13;

            *(ptr.add(4) as *mut u64) = self.amount;

            *ptr.add(12) = self.decimals;
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { &instruction_data.assume_init() },
        };

        invoke_signed(
            &instruction, 
            &[self.token, self.delegate, self.authority], 
            signers)
    }
}