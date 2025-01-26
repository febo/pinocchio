use core::{marker::PhantomData, ops::Deref};

use pinocchio::account_info::{Account, AccountInfo};

/// An adapter for [`solana_account_info::AccountInfo`] that allows
/// to access the account data through an [`AccountInfo`].
pub struct AccountInfoAdapter<'a> {
    inner: AccountInfo,

    /// The raw pointer referenced  by account is only valid while the `&'a AccountInfo<'a>`
    /// lives. Instead of holding a reference to the actual `AccountInfo<'a>`, which would
    /// increase the size of the type, we claim to hold a reference without actually holding
    /// one using a `PhantomData<&'a AccountInfo<'a>>`.
    _account_info: PhantomData<&'a solana_account_info::AccountInfo<'a>>,
}

impl<'a> AccountInfoAdapter<'a> {
    /// Create a new `AccountInfoAdapter` from a `solana_account_info::AccountInfo`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it extracts a raw pointer from the the
    /// `solana_account_info::AccountInfo` and creates a `AccountInfo` from it.
    /// This is only valid for `AccountInfo` created by the runtime, which have
    /// a stable layout.
    ///
    /// Once the `AccountInfoAdapter` is created, it is assumed that any access to
    /// the account data and lamports must be done through the `AccountInfoAdapter`.
    pub unsafe fn new(account_info: &solana_account_info::AccountInfo) -> Self {
        // Extract the pointer to the start of the `Account`.
        let ptr = account_info.key as *const _ as *mut u8;
        // Given that an `AccountInfo` is created by the runtime and has a
        // stable layout, we can assume that the `Account` is located 8 bytes
        // before its key.
        Self {
            inner: AccountInfo::new(ptr.sub(8) as *mut Account),
            _account_info: PhantomData,
        }
    }
}

impl<'a> Deref for AccountInfoAdapter<'a> {
    type Target = AccountInfo;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
