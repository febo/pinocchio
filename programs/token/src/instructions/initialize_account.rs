use pinocchio::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
};

const RENT_ID: Pubkey =
    pinocchio_pubkey::declare_pubkey!("SysvarRent111111111111111111111111111111111");

/// Initializes a new account to hold tokens.
///
/// ### Accounts:
///   0. `[WRITE]`  The account to initialize.
///   1. `[]` The mint this account will be associated with.
///   2. `[]` The new account's owner/multisignature.
///   3. `[]` Rent sysvar
pub struct InitializeAccount<'a> {
    /// The account to initialize.
    pub account: &'a AccountInfo,
    /// The mint this account will be associated with.
    pub mint: &'a AccountInfo,
    /// The new account's owner/multisignature.
    pub owner: &'a AccountInfo,
}

impl<'a> InitializeAccount<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas = [
            AccountMeta::writable(self.account.key()),
            AccountMeta::readonly(self.mint.key()),
            AccountMeta::readonly(self.owner.key()),
            AccountMeta::readonly(&RENT_ID),
        ];

        // Instruction data layout:
        // [0..1]: u8 - Instruction tag (1 for InitializeAccount)

        // Build the instruction data
        let instruction_data = [1u8]; // InitializeAccount instruction tag

        // Create the instruction
        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        // Prepare the account infos
        let account_infos = &[self.account, self.mint, self.owner];

        // Invoke the instruction
        invoke_signed(&instruction, account_infos, signers)
    }
}
