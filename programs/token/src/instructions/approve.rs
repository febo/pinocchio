use pinocchio::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
};

/// Approves a delegate. A delegate is given the authority over tokens on behalf of the source account's owner.
///
/// ### Accounts:
///   0. `[WRITE]` The source account.
///   1. `[]` The delegate.
///   2. `[SIGNER]` The source account owner.
pub struct Approve<'a> {
    /// The source account.
    pub source: &'a AccountInfo,
    /// The delegate account.
    pub delegate: &'a AccountInfo,
    /// The owner of the source account.
    pub owner: &'a AccountInfo,
    /// The amount of tokens the delegate is approved for.
    pub amount: u64,
}

impl<'a> Approve<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas = [
            AccountMeta::writable(self.source.key()),
            AccountMeta::readonly(self.delegate.key()),
            AccountMeta::readonly_signer(self.owner.key()),
        ];

        // Instruction data layout:
        // [0..1]:   u8        Instruction tag (4 for Approve)
        // [1..9]:  u64       Amount (in little endian)

        // Build the instruction data without using offsets, since the layout is static
        let mut instruction_data = [0u8; 9];
        instruction_data[0] = 4; // Approve instruction tag
        instruction_data[1..9].copy_from_slice(&self.amount.to_le_bytes());

        // Create the instruction
        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        // Invoke the instruction
        invoke_signed(
            &instruction,
            &[self.source, self.delegate, self.owner],
            signers,
        )
    }
}
