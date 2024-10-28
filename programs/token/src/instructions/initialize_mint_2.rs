use core::mem::MaybeUninit;

use pinocchio::{
    account_info::AccountInfo, instruction::{AccountMeta, Instruction, Signer}, program::invoke_signed, pubkey::Pubkey, ProgramResult
};

/// Initialize a new mint.
///
/// ### Accounts:
///   0. `[WRITABLE]` Mint account
///   1. `[]` Rent sysvar
pub struct InitilizeMint<'a> {
    /// Mint Account.
    pub mint: &'a AccountInfo,

    /// Rent sysvar Account.
    pub rent_sysvar: &'a AccountInfo,

    /// Decimals.
    pub decimals: u8,

    /// Mint Authority.
    pub mint_authority: Pubkey,

    /// Freeze Authority.
    pub freeze_authority: Option<Pubkey>


}

impl<'a> InitilizeMint<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // account metadata
        let account_metas: [AccountMeta; 2] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::readonly(self.rent_sysvar.key()),
        ];

        // instruction data
        // -  [0..4]: instruction discriminator
        // -  [4]: decimals
        // -  [5..37] mint_authority
        // -  [37..70] freeze_authority
        let mut instruction_data = MaybeUninit::<[u8; 12]>::uninit();

        // data
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;

            *(ptr as *mut u32) = 20;

            *ptr.add(4) = self.decimals;

            *(ptr.add(5) as *mut Pubkey) = self.mint_authority;

            if self.freeze_authority.is_some() {
                *(ptr.add(37) as *mut  u32) = 1;
                *(ptr.add(41) as *mut Pubkey) = self.freeze_authority.unwrap_unchecked();
            } else { 
                *(ptr.add(37) as *mut [u8; 36]) = [0;36];
            }
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { &instruction_data.assume_init() },
        };

        invoke_signed(
            &instruction, 
            &[self.mint, self.rent_sysvar], 
            signers)
    }
}