use core::mem::MaybeUninit;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

/// Approves a delegate.
///
/// ### Accounts:
///   0. `[WRITE]` The token account.
///   1. `[]` The delegate.
///   2. `[SIGNER]` The source account owner.
pub struct Approve<'a> {
    /// Source Account.
    pub token: &'a AccountInfo,
    /// Delegate Account
    pub delegate: &'a AccountInfo,
    /// Source Owner Account
    pub authority: &'a AccountInfo,
    /// Amount
    pub amount: u64,
}

impl<'a> Approve<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.token.key()),
            AccountMeta::readonly(self.delegate.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..9]: amount (8 bytes, u64)
        let mut instruction_data = MaybeUninit::<[u8; 9]>::uninit();

        // Populate data
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            // Set discriminator as u8 at offset [0]
            *ptr = 4;
            // Set amount as u64 at offset [1..9]
            *(ptr.add(1) as *mut u64) = self.amount;
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { &instruction_data.assume_init() },
        };

        invoke_signed(
            &instruction,
            &[self.token, self.delegate, self.authority],
            signers,
        )
    }
}
