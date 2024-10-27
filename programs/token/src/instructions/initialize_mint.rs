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
    pub freeze_authority: COption<Pubkey>


}

impl<'a> InitilizeMint2<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 1] = [
            AccountMeta::writable(self.mint.key()),
        ];

        // instruction data
        // -  [0..4]: instruction discriminator
        // -  [4..5]: decimals
        // -  [5..37] mint_authority
        // -  [37..70] freeze_authority
        let mut instruction_data = MaybeUninit::<[u8; 12]>::uninit();

        // data
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;

            *(ptr as *mut u32) = 0;

            *(ptr.add(4) as *mut u8) = self.decimals;

            *(ptr.add(5) as *mut Pubkey) = self.mint_authority;

            *(ptr.add(37) as *mut  COption<Pubkey>) = self.freeze_authority;
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { &instruction_data.assume_init() },
        };

        invoke_signed(
            &instruction, 
            &[self.mint], 
            signers)
    }
}