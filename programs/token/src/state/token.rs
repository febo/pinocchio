use super::AccountState;
use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

pub struct TokenAccount(*const u8);

pub struct Token {
    
}

impl TokenAccount {
    pub const LEN: usize = 165;

    pub fn from_account_info_unchecked(account_info: &AccountInfo) -> Self {
        unsafe { Self(account_info.borrow_data_unchecked().as_ptr()) }
    }

    pub fn from_account_info(account_info: &AccountInfo) -> Self {
        assert_eq!(account_info.data_len(), Self::LEN);
        assert_eq!(account_info.owner(), &crate::ID);
        unsafe { Self(account_info.borrow_data_unchecked().as_ptr()) }
    }

    pub fn mint(&self) -> Pubkey {
        unsafe { *(self.0 as *const Pubkey) }
    }

    pub fn authority(&self) -> Pubkey {
        unsafe { *(self.0.add(32) as *const Pubkey) }
    }

    pub fn amount(&self) -> u64 {
        unsafe { *(self.0.add(64) as *const u64) }
    }

    pub fn has_delegate(&self) -> bool {
        unsafe { *(self.0.add(72) as *const bool) }
    }

    pub fn delegate(&self) -> Pubkey {
        unsafe { *(self.0.add(76) as *const Pubkey) }
    }

    pub fn optional_delegate(&self) -> Option<Pubkey> {
        if self.has_delegate() {
            Some(self.delegate())
        } else {
            None
        }
    }

    pub fn state(&self) -> AccountState {
        unsafe { *(self.0.add(108) as *const AccountState) }
    }

    pub fn is_native(&self) -> bool {
        unsafe { *(self.0.add(109) as *const bool) }
    }

    pub fn native_amount(&self) -> u64 {
        unsafe { *(self.0.add(113) as *const u64) }
    }

    pub fn optional_native_amount(&self) -> Option<u64> {
        if self.is_native() {
            Some(self.native_amount())
        } else {
            None
        }
    }

    pub fn delegated_amount(&self) -> u64 {
        unsafe { *(self.0.add(121) as *const u64) }
    }

    pub fn has_close_authority(&self) -> bool {
        unsafe { *(self.0.add(129) as *const bool) }
    }

    pub fn close_authority(&self) -> Pubkey {
        unsafe { *(self.0.add(133) as *const Pubkey) }
    }

    pub fn optional_close_authority(&self) -> Option<Pubkey> {
        if self.has_close_authority() {
            Some(self.close_authority())
        } else {
            None
        }
    }
}
