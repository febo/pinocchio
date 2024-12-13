use pinocchio::{
    account_info::{AccountInfo, Ref},
    program_error::ProgramError,
};

use crate::{state::AccountState, ID};

pub struct DefaultAccountState {
    pub state: AccountState,
}

impl DefaultAccountState {
    /// The length of the `MemoTranfer` account data.
    pub const LEN: usize = core::mem::size_of::<DefaultAccountState>();

    /// Return a `TransferFeeConfig` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<Ref<DefaultAccountState>, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner() != &ID {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(Ref::map(account_info.try_borrow_data()?, |data| unsafe {
            Self::from_bytes(data)
        }))
    }

    /// Return a `MemoTransfer` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, but does not
    /// perform the borrow check.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to borrow the account data – e.g., there are
    /// no mutable borrows of the account data.
    #[inline]
    pub unsafe fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&Self, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner() != &ID {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(Self::from_bytes(account_info.borrow_data_unchecked()))
    }

    /// Return a `TransferFeeConfig` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `TransferFeeConfig`.
    #[inline(always)]
    pub unsafe fn from_bytes(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const &DefaultAccountState)
    }
}

pub struct InitializeDefaultAccountState<'a> {
    /// The mint to initialize
    pub mint: &'a AccountInfo,
    /// Default account state
    pub state: u8,
}

pub struct UpdateDefaultAccountState<'a> {
    /// The mint to update
    pub mint: &'a AccountInfo,
    /// The mint's freeze authority
    pub mint_freeze_authority: &'a AccountInfo,
    /// The new state
    pub new_state: u8,
}
