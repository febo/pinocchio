//! Data structures to represent account information.

use core::{ptr::NonNull, slice::from_raw_parts_mut};

use crate::{program_error::ProgramError, pubkey::Pubkey, syscalls::sol_memset_};

/// Maximum number of bytes a program may add to an account during a
/// single realloc.
pub const MAX_PERMITTED_DATA_INCREASE: usize = 1_024 * 10;

/// Raw account data.
///
/// This data is wrapped in an `AccountInfo` struct, which provides safe access
/// to the data.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct Account {
    /// Borrow state of the account data.
    ///
    /// 0) We reuse the duplicate flag for this. We set it to 0b0000_0000.
    /// 1) We use the first four bits to track state of lamport borrow
    /// 2) We use the second four bits to track state of data borrow
    ///
    /// 4 bit state: [1 bit mutable borrow flag | u3 immmutable borrow flag]
    /// This gives us up to 7 immutable borrows. Note that does not mean 7
    /// duplicate account infos, but rather 7 calls to borrow lamports or
    /// borrow data across all duplicate account infos.
    pub(crate) borrow_state: u8,

    /// Indicates whether the transaction was signed by this account.
    is_signer: u8,

    /// Indicates whether the account is writable.
    is_writable: u8,

    /// Indicates whether this account represents a program.
    executable: u8,

    /// Account's original data length when it was serialized for the
    /// current program invocation.
    ///
    /// The value of this field is lazily initialized to the current data length.
    /// On first access, the original data length will be 0. The caller should
    /// ensure that the original data length is set to the current data length for
    /// subsequence access.
    ///
    /// The value of this field is currently only used for `realloc`.
    original_data_len: [u8; 4],

    /// Public key of the account
    key: Pubkey,

    /// Program that owns this account
    owner: Pubkey,

    /// The lamports in the account.  Modifiable by programs.
    lamports: u64,

    /// Length of the data.
    pub(crate) data_len: u64,
}

// Convenience macro to get the original data length from the account – the value will
// be zero on first access.
macro_rules! get_original_data_len {
    ( $self:expr ) => {
        unsafe { *(&(*$self).original_data_len as *const _ as *const u32) as usize }
    };
}

// Convenience macro to set the original data length in the account.
macro_rules! set_original_data_len {
    ( $self:expr, $len:expr ) => {
        unsafe {
            *(&mut (*$self).original_data_len) = u32::to_le_bytes($len as u32);
        }
    };
}

/// Wrapper struct for an `Account`.
///
/// This struct provides safe access to the data in an `Account`. It is also
/// used to track borrows of the account data and lamports, given that an
/// account can be "shared" across multiple `AccountInfo` instances.
#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct AccountInfo {
    /// Raw (pointer to) account data.
    ///
    /// Note that this is a pointer can be shared across multiple `AccountInfo`.
    pub(crate) raw: *mut Account,
}

impl AccountInfo {
    /// Public key of the account.
    #[inline(always)]
    pub fn key(&self) -> &Pubkey {
        unsafe { &(*self.raw).key }
    }

    /// Program that owns this account.
    #[inline(always)]
    pub fn owner(&self) -> &Pubkey {
        unsafe { &(*self.raw).owner }
    }

    /// Indicates whether the transaction was signed by this account.
    #[inline(always)]
    pub fn is_signer(&self) -> bool {
        unsafe { (*self.raw).is_signer != 0 }
    }

    /// Indicates whether the account is writable.
    #[inline(always)]
    pub fn is_writable(&self) -> bool {
        unsafe { (*self.raw).is_writable != 0 }
    }

    /// Indicates whether this account represents a program.
    ///
    /// Program accounts are always read-only.
    #[inline(always)]
    pub fn executable(&self) -> bool {
        unsafe { (*self.raw).executable != 0 }
    }

    /// Returns the size of the data in the account.
    #[inline(always)]
    pub fn data_len(&self) -> usize {
        unsafe { (*self.raw).data_len as usize }
    }

    /// Indicates whether the account data is empty.
    ///
    /// An account is considered empty if the data length is zero.
    #[inline(always)]
    pub fn data_is_empty(&self) -> bool {
        self.data_len() == 0
    }

    /// Changes the owner of the account.
    #[allow(invalid_reference_casting)]
    pub fn assign(&self, new_owner: &Pubkey) {
        unsafe {
            core::ptr::write_volatile(&(*self.raw).owner as *const _ as *mut Pubkey, *new_owner);
        }
    }

    /// Returns a read-only reference to the lamports in the account.
    ///
    /// # Safety
    ///
    /// This does not check or modify the 4-bit refcell. Useful when instruction
    /// has verified non-duplicate accounts.
    pub unsafe fn unchecked_borrow_lamports(&self) -> &u64 {
        &(*self.raw).lamports
    }

    /// Returns a mutable reference to the lamports in the account.
    ///
    /// # Safety
    ///
    /// This does not check or modify the 4-bit refcell. Useful when instruction
    /// has verified non-duplicate accounts.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn unchecked_borrow_mut_lamports(&self) -> &mut u64 {
        &mut (*self.raw).lamports
    }

    /// Returns a read-only reference to the data in the account.
    ///
    /// # Safety
    ///
    /// This does not check or modify the 4-bit refcell. Useful when instruction
    /// has verified non-duplicate accounts.
    pub unsafe fn unchecked_borrow_data(&self) -> &[u8] {
        core::slice::from_raw_parts(self.data_ptr(), self.data_len())
    }

    /// Returns a mutable reference to the data in the account.
    ///
    /// # Safety
    ///
    /// This does not check or modify the 4-bit refcell. Useful when instruction
    /// has verified non-duplicate accounts.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn unchecked_borrow_mut_data(&self) -> &mut [u8] {
        core::slice::from_raw_parts_mut(self.data_ptr(), self.data_len())
    }

    /// Tries to get a read-only reference to the lamport field, failing if the
    /// field is already mutable borrowed or if 7 borrows already exist.
    pub fn try_borrow_lamports(&self) -> Result<Ref<u64>, ProgramError> {
        let borrow_state = unsafe { &mut (*self.raw).borrow_state };

        // check if mutable borrow is already taken
        if *borrow_state & 0b_1000_0000 != 0 {
            return Err(ProgramError::AccountBorrowFailed);
        }

        // check if we have reached the max immutable borrow count
        if *borrow_state & 0b_0111_0000 == 0b_0111_0000 {
            return Err(ProgramError::AccountBorrowFailed);
        }

        // increment the immutable borrow count
        *borrow_state += 1 << LAMPORTS_SHIFT;

        // return the reference to lamports
        Ok(Ref {
            value: unsafe { &(*self.raw).lamports },
            state: unsafe { NonNull::new_unchecked(&mut (*self.raw).borrow_state) },
            borrow_shift: LAMPORTS_SHIFT,
        })
    }

    /// Tries to get a read only reference to the lamport field, failing if the field
    /// is already borrowed in any form.
    pub fn try_borrow_mut_lamports(&self) -> Result<RefMut<u64>, ProgramError> {
        let borrow_state = unsafe { &mut (*self.raw).borrow_state };

        // check if any borrow (mutable or immutable) is already taken for lamports
        if *borrow_state & 0b_1111_0000 != 0 {
            return Err(ProgramError::AccountBorrowFailed);
        }

        // set the mutable lamport borrow flag
        *borrow_state |= 0b_1000_0000;

        // return the mutable reference to lamports
        Ok(RefMut {
            value: unsafe { &mut (*self.raw).lamports },
            state: unsafe { NonNull::new_unchecked(&mut (*self.raw).borrow_state) },
            borrow_mask: LAMPORTS_MASK,
        })
    }

    /// Tries to get a read only reference to the data field, failing if the field
    /// is already mutable borrowed or if 7 borrows already exist.
    pub fn try_borrow_data(&self) -> Result<Ref<[u8]>, ProgramError> {
        let borrow_state = unsafe { &mut (*self.raw).borrow_state };

        // check if mutable data borrow is already taken (most significant bit
        // of the data_borrow_state)
        if *borrow_state & 0b_0000_1000 != 0 {
            return Err(ProgramError::AccountBorrowFailed);
        }

        // check if we have reached the max immutable data borrow count (7)
        if *borrow_state & 0b0111 == 0b0111 {
            return Err(ProgramError::AccountBorrowFailed);
        }

        // increment the immutable data borrow count
        *borrow_state += 1;

        // return the reference to data
        Ok(Ref {
            value: unsafe { core::slice::from_raw_parts(self.data_ptr(), self.data_len()) },
            state: unsafe { NonNull::new_unchecked(&mut (*self.raw).borrow_state) },
            borrow_shift: DATA_SHIFT,
        })
    }

    /// Tries to get a read only reference to the data field, failing if the field
    /// is already borrowed in any form.
    pub fn try_borrow_mut_data(&self) -> Result<RefMut<[u8]>, ProgramError> {
        let borrow_state = unsafe { &mut (*self.raw).borrow_state };

        // check if any borrow (mutable or immutable) is already taken for data
        if *borrow_state & 0b_0000_1111 != 0 {
            return Err(ProgramError::AccountBorrowFailed);
        }

        // set the mutable data borrow flag
        *borrow_state |= 0b0000_1000;

        // return the mutable reference to data
        Ok(RefMut {
            value: unsafe { from_raw_parts_mut(self.data_ptr(), self.data_len()) },
            state: unsafe { NonNull::new_unchecked(&mut (*self.raw).borrow_state) },
            borrow_mask: DATA_MASK,
        })
    }

    /// Realloc the account's data and optionally zero-initialize the new
    /// memory.
    ///
    /// Note:  Account data can be increased within a single call by up to
    /// `solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE` bytes.
    ///
    /// Note: Memory used to grow is already zero-initialized upon program
    /// entrypoint and re-zeroing it wastes compute units.  If within the same
    /// call a program reallocs from larger to smaller and back to larger again
    /// the new space could contain stale data.  Pass `true` for `zero_init` in
    /// this case, otherwise compute units will be wasted re-zero-initializing.
    ///
    /// # Safety
    ///
    /// This method makes assumptions about the layout and location of memory
    /// referenced by `AccountInfo` fields. It should only be called for
    /// instances of `AccountInfo` that were created by the runtime and received
    /// in the `process_instruction` entrypoint of a program.
    pub fn realloc(&self, new_len: usize, zero_init: bool) -> Result<(), ProgramError> {
        let mut data = self.try_borrow_mut_data()?;
        let current_len = data.len();

        // return early if length hasn't changed
        if new_len == current_len {
            return Ok(());
        }

        let original_len = match get_original_data_len!(self.raw) {
            len if len > 0 => len,
            _ => {
                set_original_data_len!(self.raw, current_len);
                current_len
            }
        };

        // return early if the length increase from the original serialized data
        // length is too large and would result in an out of bounds allocation
        if new_len.saturating_sub(original_len) > MAX_PERMITTED_DATA_INCREASE {
            return Err(ProgramError::InvalidRealloc);
        }

        // realloc
        unsafe {
            let data_ptr = data.as_mut_ptr();
            // set new length in the serialized data
            *(data_ptr.offset(-8) as *mut u64) = new_len as u64;
            // recreate the local slice with the new length
            data.value = from_raw_parts_mut(data_ptr, new_len);
        }

        if zero_init {
            let len_increase = new_len.saturating_sub(current_len);
            if len_increase > 0 {
                unsafe {
                    sol_memset_(
                        &mut data[original_len..] as *mut _ as *mut u8,
                        0,
                        len_increase as u64,
                    );
                }
            }
        }

        Ok(())
    }

    /// Returns the memory address of the account data.
    fn data_ptr(&self) -> *mut u8 {
        unsafe { (self.raw as *const _ as *mut u8).add(core::mem::size_of::<Account>()) }
    }
}

/// Bytes to shift to get to the borrow state of lamports.
const LAMPORTS_SHIFT: u8 = 4;

/// Bytes to shift to get to the borrow state of data.
const DATA_SHIFT: u8 = 0;

/// Reference to account data or lamports with checked borrow rules.
pub struct Ref<'a, T: ?Sized> {
    value: &'a T,
    state: NonNull<u8>,
    /// Indicates the type of borrow (lamports or data) by representing the
    /// shift amount.
    borrow_shift: u8,
}

impl<'a, T: ?Sized> core::ops::Deref for Ref<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T: ?Sized> Drop for Ref<'a, T> {
    // decrement the immutable borrow count
    fn drop(&mut self) {
        unsafe { *self.state.as_mut() -= 1 << self.borrow_shift };
    }
}

/// Mask representing the mutable borrow flag for lamports.
const LAMPORTS_MASK: u8 = 0b_0111_1111;

/// Mask representing the mutable borrow flag for data.
const DATA_MASK: u8 = 0b_1111_0111;

/// Mutable reference to account data or lamports with checked borrow rules.
pub struct RefMut<'a, T: ?Sized> {
    value: &'a mut T,
    state: NonNull<u8>,
    /// Indicates the type of borrow (lamports or data) by representing the
    /// mutable borrow mask.
    borrow_mask: u8,
}

impl<'a, T: ?Sized> core::ops::Deref for RefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}
impl<'a, T: ?Sized> core::ops::DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut <Self as core::ops::Deref>::Target {
        self.value
    }
}

impl<'a, T: ?Sized> Drop for RefMut<'a, T> {
    // unset the mutable borrow flag
    fn drop(&mut self) {
        unsafe { *self.state.as_mut() &= self.borrow_mask };
    }
}
