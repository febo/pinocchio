use crate::ID;

use pinocchio::{account_info::{AccountInfo, Ref}, program_error::ProgramError, pubkey::Pubkey};
pub struct Mint(*const u8);

impl Mint {
    pub const LEN: usize = 82;

    /// Performs owner and length validation on `AccountInfo` and returns a `Ref<T>` for safe borrowing.
    pub fn from_account_info(account_info: &AccountInfo) -> Result<Ref<Mint>, ProgramError> {
        if account_info.data_len() != Self::LEN { return Err(ProgramError::InvalidAccountData) }
        if account_info.owner() != &ID { return Err(ProgramError::InvalidAccountData) }
        Ok(Ref::map(account_info.try_borrow_data()?, |data| {
            unsafe { &*(data.as_ptr() as *const Mint) }
        }))
    }

    /// # Safety
    /// Performs owner and length validation on `AccountInfo` but performs unchecked borrowing and 
    /// returns a `T` directly.
    #[inline(always)]
    pub unsafe fn from_account_info_unchecked(account_info: &AccountInfo) -> Result<Mint, ProgramError> {
        if account_info.data_len() != Self::LEN { return Err(ProgramError::InvalidAccountData) }
        if account_info.owner() != &ID { return Err(ProgramError::InvalidAccountData) }
        Ok(Self::from_bytes(account_info.borrow_data_unchecked().as_ref()))
    }

    /// # Safety
    /// Constructs a `T` directly from a byte slice. The caller must ensure that `bytes` contains a 
    /// valid representation of `T`.
    pub unsafe fn from_bytes(bytes: &[u8]) -> Self {
        core::ptr::read(bytes.as_ptr() as *const Mint)
    }

    #[inline(always)]
    pub fn has_mint_authority(&self) -> bool {
        unsafe { *(self.0 as *const bool) }
    }

    pub fn mint_authority(&self) -> Option<Pubkey> {
        if self.has_mint_authority() {
            Some(self.mint_authority_unchecked())
        } else {
            None
        }
    }

    /// Use this when you know the account will have a mint authority and you want to skip the Option check.
    #[inline(always)]
    pub fn mint_authority_unchecked(&self) -> Pubkey {
        unsafe { *(self.0.add(4) as *const Pubkey) }
    }

    pub fn supply(&self) -> u64 {
        unsafe { core::ptr::read_unaligned(self.0.add(36) as *const u64) }
    }

    pub fn decimals(&self) -> u8 {
        unsafe { *self.0.add(44) }
    }

    pub fn is_initialized(&self) -> bool {
        unsafe { *(self.0.add(45) as *const bool) }
    }

    #[inline(always)]
    pub fn has_freeze_authority(&self) -> bool {
        unsafe { *(self.0.add(46) as *const bool) }
    }

    pub fn freeze_authority(&self) -> Option<Pubkey> {
        if self.has_freeze_authority() {
            Some(self.freeze_authority_unchecked())
        } else {
            None
        }
    }

    /// Use this when you know the account will have a freeze authority and you want to skip the Option check.
    #[inline(always)]
    pub fn freeze_authority_unchecked(&self) -> Pubkey {
        unsafe { *(self.0.add(50) as *const Pubkey) }
    }
}
