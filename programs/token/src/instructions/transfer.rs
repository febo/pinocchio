use pinocchio::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
};

/// Transfers tokens from one account to another either directly or via a delegate.
///
/// ### Accounts:
///   0. `[WRITE]` The source account.
///   1. `[WRITE]` The destination account.
///   2. `[SIGNER]` The source account's owner/delegate.
pub struct Transfer<'a> {
    /// The source account.
    pub source: &'a AccountInfo,
    /// The destination account.
    pub destination: &'a AccountInfo,
    /// The owner or delegate of the source account.
    pub authority: &'a AccountInfo,
    /// The amount of tokens to transfer.
    pub amount: u64,
}

impl<'a> Transfer<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas = [
            AccountMeta::writable(self.source.key()),
            AccountMeta::writable(self.destination.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data layout:
        // [0..1]:   u8   - Instruction tag (3 for Transfer)
        // [1..9]:  u64  - Amount (in little endian)

        // Build the instruction data without using offsets since the layout is static
        let mut instruction_data = [0u8; 9];
        instruction_data[0] = 3; // Transfer instruction tag
        instruction_data[1..9].copy_from_slice(&self.amount.to_le_bytes());

        // Create the instruction
        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        // Prepare the account infos
        let account_infos = &[self.source, self.destination, self.authority];

        // Invoke the instruction
        invoke_signed(&instruction, account_infos, signers)
    }
}
