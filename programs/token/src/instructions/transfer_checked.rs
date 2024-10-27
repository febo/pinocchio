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
///   0. `[WRITE]` The source account.
///   1. `[]` The token mint.
///   2. `[WRITE]` The destination account.
///   3. `[SIGNER]` The source account's owner/delegate.
pub struct TransferChecked<'a> {
    /// Sender account.
    pub from: &'a AccountInfo,

    /// Mint Account
    pub mint: &'a AccountInfo,

    /// Recipient account.
    pub to: &'a AccountInfo,

    /// Authority account.
    pub authority: &'a AccountInfo,

    /// Amount of microtokens to transfer.
    pub amount: u64,

    /// Decimal for the Token
    pub decimals: u8,
}

impl<'a> TransferChecked<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.from.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::writable(self.to.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // instruction data
        // -  [0..4 ]: instruction discriminator
        // -  [4..12]: amount
        // -  [12..13]: decimals
        let mut instruction_data = MaybeUninit::<[u8; 12]>::uninit();

        unsafe {
            // Get a mutable pointer to the instruction_data
            let ptr = instruction_data.as_mut_ptr() as *mut u8;

            // Write 3 as u32 to the first 4 bytes
            *(ptr as *mut u32) = 12;

            // Write self.amount as u64 to the next 8 bytes
            *(ptr.add(4) as *mut u64) = self.amount;

            *(ptr.add(12) as *mut u8) = self.decimals;
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { &instruction_data.assume_init() },
        };

        invoke_signed(&instruction, &[self.from, self.to, self.authority], signers)
    }
}
