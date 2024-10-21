use pinocchio::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    sysvars::rent::Rent,
};

const RENT_ID: Pubkey =
    pinocchio_pubkey::declare_pubkey!("SysvarRent111111111111111111111111111111111");

/// Initializes a new mint.
///
/// ### Accounts:
///   0. `[WRITE]` The mint account to initialize.
///   1. `[]` Rent sysvar
pub struct InitializeMint<'a> {
    /// The mint account to initialize.
    pub mint: &'a AccountInfo,
    /// The rent sysvar.
    pub rent: &'a AccountInfo,
    /// Number of base 10 digits to the right of the decimal place.
    pub decimals: u8,
    /// The authority/multisignature to mint tokens.
    pub mint_authority: &'a Pubkey,
    /// The freeze authority/multisignature of the mint (optional).
    pub freeze_authority: Option<&'a Pubkey>,
}

impl<'a> InitializeMint<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly(&RENT_ID),
        ];

        // Instruction data layout:
        // [0..1]      = u8         Instruction tag (0 for InitializeMint)
        // [1..2]      = u8         Decimals
        // [2..34]     = [u8; 32]   Mint authority pubkey
        // [34..67]    = [u8; 32]   Freeze authority pubkey (if Some)

        // Build the instruction data without using offsets, since the layout is static
        let mut instruction_data = [0u8; 67];

        // Instruction tag
        instruction_data[0] = 0; // InitializeMint instruction tag

        // Decimals
        instruction_data[1] = self.decimals;

        // Mint authority
        instruction_data[2..34].copy_from_slice(self.mint_authority.as_ref());

        // Freeze authority (COption<Pubkey>)
        match self.freeze_authority {
            Some(freeze_authority) => {
                instruction_data[34] = 1; // COption::Some
                instruction_data[35..67].copy_from_slice(freeze_authority.as_ref());
            }
            None => {
                instruction_data[34] = 0; // COption::None
            }
        }

        // Create the instruction
        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        // Invoke the instruction
        invoke_signed(&instruction, &[self.mint], signers)
    }
}
