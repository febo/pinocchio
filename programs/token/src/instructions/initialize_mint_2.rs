use core::mem::MaybeUninit;

use pinocchio::{
    account_info::AccountInfo, instruction::{AccountMeta, Instruction, Signer}, program::invoke_signed, pubkey::Pubkey, ProgramResult
};

/// Initialize a new mint.
///
/// ### Accounts:
///   0. `[WRITABLE]` Mint account
pub struct InitilizeMint2<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,
    /// Decimals.
    pub decimals: u8,
    /// Mint Authority.
    pub mint_authority: Pubkey,
    /// Freeze Authority.
    pub freeze_authority: Option<Pubkey>,
}

impl<'a> InitilizeMint2<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 1] = [
            AccountMeta::writable(self.mint.key()),
        ];

        // Instruction data layout:
        // -  [0]: instruction discriminator 
        // -  [1]: decimals 
        // -  [2..34]: mint_authority 
        // -  [34..38]: freeze_authority presence flag 
        // -  [38..70]: freeze_authority 
        let mut instruction_data = MaybeUninit::<[u8; 70]>::uninit();

        // Populate data
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            // Set discriminator as u8 at offset [0]
            *ptr = 20;
            // Set decimals as u8 at offset [1]
            *ptr.add(1) = self.decimals;
            // Set mint_authority as Pubkey at offset [2..34]
            *(ptr.add(2) as *mut Pubkey) = self.mint_authority;
            // Set COption & freeze_authority at offset [34..70]
            if let Some(freeze_auth) = self.freeze_authority {
                *(ptr.add(34) as *mut  u32) = 1;
                *(ptr.add(38) as *mut Pubkey) = freeze_auth;
            } else {
                *(ptr.add(34) as *mut [u8; 36]) = [0; 36];
            }
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { &instruction_data.assume_init() },
        };

        invoke_signed(
            &instruction, 
            &[self.mint], 
            signers
        )
    }
}
