use core::mem::MaybeUninit;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    ProgramResult,
};

/// Transfer Tokens from one Token Account to another.
///
/// ### Accounts:
///   0. `[WRITE]` Sender account
///   1. `[WRITE]` Recipient account
///   2. `[SIGNER]` Authority account
pub struct Transfer<'a> {
    /// Sender account.
    pub from: &'a AccountInfo,

    /// Recipient account.
    pub to: &'a AccountInfo,

    /// Authority account.
    pub authority: &'a AccountInfo,

    /// Amount of microtokens to transfer.
    pub amount: u64,
}

impl<'a> Transfer<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.from.key()),
            AccountMeta::writable(self.to.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // instruction data
        // -  [0..4 ]: instruction discriminator
        // -  [4..12]: amount
        let mut instruction_data = MaybeUninit::<[u8; 12]>::uninit();

        unsafe {
            // Get a mutable pointer to the instruction_data
            let ptr = instruction_data.as_mut_ptr() as *mut u8;

            // Write 3 as u32 to the first 4 bytes
            *(ptr as *mut u32) = 3;

            // Write self.amount as u64 to the next 8 bytes
            *(ptr.add(4) as *mut u64) = self.amount;
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { &instruction_data.assume_init() },
        };

        invoke_signed(&instruction, &[self.from, self.to, self.authority], signers)
    }
}
