//! Macros and functions for defining the program entrypoint and setting up
//! global handlers.

use core::{alloc::Layout, mem::size_of, ptr::null_mut, slice::from_raw_parts};

use crate::{
    account_info::{Account, AccountInfo, MAX_PERMITTED_DATA_INCREASE},
    pubkey::Pubkey,
    BPF_ALIGN_OF_U128, NON_DUP_MARKER,
};

/// Start address of the memory region used for program heap.
pub const HEAP_START_ADDRESS: u64 = 0x300000000;

/// Length of the heap memory region used for program heap.
pub const HEAP_LENGTH: usize = 32 * 1024;

#[deprecated(
    since = "0.6.0",
    note = "Use `ProgramResult` from the crate root instead"
)]
/// The result of a program execution.
pub type ProgramResult = super::ProgramResult;

#[deprecated(since = "0.6.0", note = "Use `SUCCESS` from the crate root instead")]
/// Return value for a successful program execution.
pub const SUCCESS: u64 = super::SUCCESS;

/// Declare the program entrypoint and set up global handlers.
///
/// The main difference from the standard `entrypoint!` macro is that this macro represents an
/// entrypoint that does not perform allocattions or copies when reading the input buffer.
///
/// This macro emits the common boilerplate necessary to begin program execution, calling a
/// provided function to process the program instruction supplied by the runtime, and reporting
/// its result to the runtime.
///
/// It also sets up a [global allocator] and [panic handler], using the [`custom_heap_default`]
/// and [`custom_panic_default`] macros.
///
/// The first argument is the name of a function with this type signature:
///
/// ```ignore
/// fn process_instruction(
///     program_id: &Pubkey,      // Public key of the account the program was loaded into
///     accounts: &[AccountInfo], // All accounts required to process the instruction
///     instruction_data: &[u8],  // Serialized instruction-specific data
/// ) -> ProgramResult;
/// ```
///
/// The second (optional) argument is the maximum number of accounts that the program is expecting.
/// A program can receive more than the specified maximum, but any account exceeding the maximum will
/// be ignored. When the maximum is not specified, the default is 64. This is currently the [maximum
/// number of accounts] that a transaction may lock in a block.
///
/// [maximum number of accounts]: https://github.com/anza-xyz/agave/blob/ccabfcf84921977202fd06d3197cbcea83742133/runtime/src/bank.rs#L3207-L3219
///
/// # Examples
///
/// Defining an entrypoint which reads up to 10 accounts and making it conditional on the
/// `bpf-entrypoint` feature. Although the `entrypoint` module is written inline in this example,
/// it is common to put it into its own file.
///
/// ```no_run
/// #[cfg(feature = "bpf-entrypoint")]
/// pub mod entrypoint {
///
///     use pinocchio::{
///         account_info::AccountInfo,
///         entrypoint,
///         msg,
///         pubkey::Pubkey,
///         ProgramResult
///     };
///
///     entrypoint!(process_instruction);
///
///     pub fn process_instruction(
///         program_id: &Pubkey,
///         accounts: &[AccountInfo],
///         instruction_data: &[u8],
///     ) -> ProgramResult {
///         msg!("Hello from my program!");
///         Ok(())
///     }
///
/// }
/// ```
#[macro_export]
macro_rules! entrypoint {
    ( $process_instruction:ident ) => {
        entrypoint!($process_instruction, { $crate::MAX_TX_ACCOUNTS });
    };
    ( $process_instruction:ident, $maximum:expr ) => {
        /// Program entrypoint.
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
            const UNINIT: core::mem::MaybeUninit<$crate::account_info::AccountInfo> =
                core::mem::MaybeUninit::<$crate::account_info::AccountInfo>::uninit();
            // create an array of uninitialized account infos
            let mut accounts = [UNINIT; $maximum];

            let (program_id, count, instruction_data) =
                $crate::entrypoint::deserialize::<$maximum>(input, &mut accounts);

            // call the program's entrypoint passing `count` account infos; we know that
            // they are initialized so we cast the pointer to a slice of `[AccountInfo]`
            match $process_instruction(
                &program_id,
                core::slice::from_raw_parts(accounts.as_ptr() as _, count),
                &instruction_data,
            ) {
                Ok(()) => $crate::SUCCESS,
                Err(error) => error.into(),
            }
        }

        $crate::custom_heap_default!();
        $crate::custom_panic_default!();
    };
}

/// Deserialize the input arguments.
///
/// This can only be called from the entrypoint function of a Solana program and with
/// a buffer that was serialized by the runtime.
#[allow(clippy::cast_ptr_alignment, clippy::missing_safety_doc)]
#[inline(always)]
pub unsafe fn deserialize<'a, const MAX_ACCOUNTS: usize>(
    input: *mut u8,
    accounts: &mut [core::mem::MaybeUninit<AccountInfo>],
) -> (&'a Pubkey, usize, &'a [u8]) {
    let mut offset: usize = 0;

    // total number of accounts present; it only process up to MAX_ACCOUNTS
    let total_accounts = *(input.add(offset) as *const u64) as usize;
    offset += core::mem::size_of::<u64>();

    let processed = if total_accounts > 0 {
        // number of accounts to process (limited to MAX_ACCOUNTS)
        let processed = core::cmp::min(total_accounts, MAX_ACCOUNTS);

        for i in 0..processed {
            let account_info: *mut Account = input.add(offset) as *mut _;

            if (*account_info).borrow_state == NON_DUP_MARKER {
                // repurpose the borrow state to track borrows
                (*account_info).borrow_state = 0b_0000_0000;

                offset += core::mem::size_of::<Account>();
                offset += (*account_info).data_len as usize;
                offset += MAX_PERMITTED_DATA_INCREASE;
                offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128);
                offset += core::mem::size_of::<u64>();

                accounts[i].write(AccountInfo { raw: account_info });
            } else {
                offset += core::mem::size_of::<u64>();
                // duplicate account, clone the original pointer
                accounts[i].write(
                    accounts[(*account_info).borrow_state as usize]
                        .assume_init_ref()
                        .clone(),
                );
            }
        }

        // process any remaining accounts to move the offset to the instruction
        // data (there is a duplication of logic but we avoid testing whether we
        // have space for the account or not)
        for _ in processed..total_accounts {
            let account_info: *mut Account = input.add(offset) as *mut _;

            if (*account_info).borrow_state == NON_DUP_MARKER {
                offset += core::mem::size_of::<Account>();
                offset += (*account_info).data_len as usize;
                offset += MAX_PERMITTED_DATA_INCREASE;
                offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128);
                offset += core::mem::size_of::<u64>();
            } else {
                offset += core::mem::size_of::<u64>();
            }
        }

        processed
    } else {
        // no accounts to process
        0
    };

    // instruction data
    let instruction_data_len = *(input.add(offset) as *const u64) as usize;
    offset += core::mem::size_of::<u64>();

    let instruction_data = { from_raw_parts(input.add(offset), instruction_data_len) };
    offset += instruction_data_len;

    // program id
    let program_id: &Pubkey = &*(input.add(offset) as *const Pubkey);

    (program_id, processed, instruction_data)
}

#[macro_export]
macro_rules! custom_panic_default {
    () => {
        /// Default panic handler.
        #[cfg(all(not(feature = "custom-panic"), target_os = "solana"))]
        #[no_mangle]
        fn custom_panic(info: &core::panic::PanicInfo<'_>) {
            // Full panic reporting.
            $crate::msg!("{}", info);
        }
    };
}

#[macro_export]
macro_rules! custom_heap_default {
    () => {
        #[cfg(all(not(feature = "custom-heap"), target_os = "solana"))]
        extern crate alloc;

        #[cfg(all(not(feature = "custom-heap"), target_os = "solana"))]
        #[global_allocator]
        static A: $crate::entrypoint::BumpAllocator = $crate::entrypoint::BumpAllocator {
            start: $crate::entrypoint::HEAP_START_ADDRESS as usize,
            len: $crate::entrypoint::HEAP_LENGTH,
        };
    };
}

/// The bump allocator used as the default rust heap when running programs.
pub struct BumpAllocator {
    pub start: usize,
    pub len: usize,
}

/// Integer arithmetic in this global allocator implementation is safe when
/// operating on the prescribed `HEAP_START_ADDRESS` and `HEAP_LENGTH`. Any
/// other use may overflow and is thus unsupported and at one's own risk.
#[allow(clippy::arithmetic_side_effects)]
unsafe impl core::alloc::GlobalAlloc for BumpAllocator {
    /// Allocates memory as a bump allocator.
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pos_ptr = self.start as *mut usize;

        let mut pos = *pos_ptr;
        if pos == 0 {
            // First time, set starting position.
            pos = self.start + self.len;
        }
        pos = pos.saturating_sub(layout.size());
        pos &= !(layout.align().wrapping_sub(1));
        if pos < self.start + size_of::<*mut u8>() {
            return null_mut();
        }
        *pos_ptr = pos;
        pos as *mut u8
    }
    #[inline]
    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        // I'm a bump allocator, I don't free
    }
}
