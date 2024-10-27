use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};
pub struct Mint(*const u8);

impl Mint {
    pub const LEN: usize = 82;

    pub fn from_account_info_unchecked(account_info: &AccountInfo) -> Self {
        unsafe { Self(account_info.borrow_data_unchecked().as_ptr()) }
    }

    pub fn from_account_info(account_info: &AccountInfo) -> Self {
        assert_eq!(account_info.data_len(), Self::LEN);
        assert_eq!(account_info.owner(), &crate::ID);
        unsafe { Self(account_info.borrow_data_unchecked().as_ptr()) }
    }

    pub fn has_mint_authority(&self) -> bool {
        unsafe { *(self.0 as *const bool) }
    }

    pub fn mint_authority(&self) -> Pubkey {
        unsafe { *(self.0.add(4) as *const Pubkey) }
    }

    pub fn supply(&self) -> u64 {
        unsafe { *(self.0.add(36) as *const u64) }
    }

    pub fn decimals(&self) -> u8 {
        unsafe { *self.0.add(44) }
    }

    pub fn is_frozen(&self) -> bool {
        unsafe { *(self.0.add(45) as *const bool) }
    }

    pub fn has_freeze_authority(&self) -> bool {
        unsafe { *(self.0.add(46) as *const bool) }
    }

    pub fn freeze_authority(&self) -> Pubkey {
        unsafe { *(self.0.add(50) as *const Pubkey) }
    }
}
