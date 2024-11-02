use crate::ID;

use super::AccountState;
use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

pub struct TokenAccount(*const u8);

pub struct Token {}

impl TokenAccount {
    pub const LEN: usize = 165;

    /// Use this when you know the AccountInfo will have the correct owner and account space
    #[inline(always)]
    pub fn from_account_info_unchecked(account_info: &AccountInfo) -> Self {
        unsafe { Self(account_info.borrow_data_unchecked().as_ptr()) }
    }

    pub fn from_account_info(account_info: &AccountInfo) -> Self {
        assert_eq!(account_info.data_len(), Self::LEN);
        assert_eq!(account_info.owner(), &ID);
        Self::from_account_info_unchecked(account_info)
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

    #[inline(always)]
    pub fn has_delegate(&self) -> bool {
        unsafe { *(self.0.add(72) as *const bool) }
    }

    pub fn delegate(&self) -> Option<Pubkey> {
        if self.has_delegate() {
            Some(self.delegate_unchecked())
        } else {
            None
        }
    }

    /// Use this when you know the account will have a delegate and want to skip the Option check.
    #[inline(always)]
    pub fn delegate_unchecked(&self) -> Pubkey {
        unsafe { *(self.0.add(76) as *const Pubkey) }
    }

    pub fn state(&self) -> AccountState {
        unsafe { *(self.0.add(108) as *const AccountState) }
    }

    pub fn is_native(&self) -> bool {
        unsafe { *(self.0.add(109) as *const bool) }
    }

    pub fn native_amount(&self) -> Option<u64> {
        if self.is_native() {
            Some(self.native_amount_unchecked())
        } else {
            None
        }
    }

    /// Use this when you know the account is native and you want to skip the Option check.
    #[inline(always)]
    pub fn native_amount_unchecked(&self) -> u64 {
        unsafe { core::ptr::read_unaligned(self.0.add(113) as *const u64) }
    }

    pub fn delegated_amount(&self) -> u64 {
        unsafe { core::ptr::read_unaligned(self.0.add(121) as *const u64) }
    }

    #[inline(always)]
    pub fn has_close_authority(&self) -> bool {
        unsafe { *(self.0.add(129) as *const bool) }
    }

    pub fn close_authority(&self) -> Option<Pubkey> {
        if self.has_close_authority() {
            Some(self.close_authority_unchecked())
        } else {
            None
        }
    }

    /// Use this when you know the account will a close authority and you want to skip the Option check.
    #[inline(always)]
    pub fn close_authority_unchecked(&self) -> Pubkey {
        unsafe { *(self.0.add(133) as *const Pubkey) }
    }
}
