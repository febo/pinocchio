use pinocchio::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction, Signer},
    msg,
    program::invoke_signed,
};

/// Creates an associated token account for the given wallet address and token mint.
/// Returns an error if the account exists.
///
/// ### Accounts:
///   0. `[WRITE, SIGNER]` Funding account (must be a system account)
///   1. `[WRITE]` Associated token account address to be created
///   2. `[]` Wallet address for the new associated token account
///   3. `[]` The token mint for the new associated token account
///   4. `[]` System program
///   5. `[]` SPL Token program
pub struct Create<'a> {
    /// Funding account (must be a system account)
    pub funding_account: &'a AccountInfo,
    /// Associated token account address to be created
    pub associated_account: &'a AccountInfo,
    /// Wallet address for the new associated token account
    pub wallet_address: &'a AccountInfo,
    /// The token mint for the new associated token account
    pub token_mint: &'a AccountInfo,
    /// System program
    pub system_program: &'a AccountInfo,
    /// SPL Token program
    pub token_program: &'a AccountInfo,
}

impl<'a> Create<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas = [
            AccountMeta::writable_signer(self.funding_account.key()),
            AccountMeta::writable(self.associated_account.key()),
            AccountMeta::readonly(self.wallet_address.key()),
            AccountMeta::readonly(self.token_mint.key()),
            AccountMeta::readonly(self.system_program.key()),
            AccountMeta::readonly(self.token_program.key()),
        ];

        // Instruction data layout:
        // [0]: u8 - Instruction tag (0 for Create)

        // Build the instruction data
        let instruction_data = [0u8]; // Create instruction tag

        // Create the instruction
        let instruction = Instruction {
            program_id: &crate::ID, // Associated Token Account program ID
            accounts: &account_metas,
            data: &instruction_data,
        };

        // Prepare the account infos
        let account_infos = &[
            self.funding_account,
            self.associated_account,
            self.wallet_address,
            self.token_mint,
            self.system_program,
            self.token_program,
        ];

        log_instruction(&instruction, account_infos);

        // Invoke the instruction
        invoke_signed(&instruction, account_infos, signers)
    }
}

pub fn log_instruction(instruction: &Instruction, account_infos: &[&AccountInfo]) {
    pinocchio::msg!("Instruction {:?}", instruction.data);

    pinocchio::msg!(
        "{:<4} {:<44} {:<44} {:<10} {:<10} {:<10}",
        "IxId",
        "Key",
        "Owner",
        "Writable",
        "Signer",
        "Invoked"
    );

    for (i, account) in instruction.accounts.iter().enumerate() {
        let is_writable = account.is_writable;
        let is_signer = account.is_signer;
        let is_invoked = account_infos[i].executable();

        pinocchio::msg!(
            "[{:<2}] {:<44} {:<44} {:<10} {:<10} {:<10}",
            i,
            bs58::encode(account.pubkey).into_string(),
            bs58::encode(account_infos[i].owner()).into_string(),
            is_writable,
            is_signer,
            is_invoked
        );
    }
}
