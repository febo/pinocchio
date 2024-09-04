use std::{alloc::Layout, mem::size_of, ptr::null_mut, slice::from_raw_parts};

use crate::{
    account_info::{Account, AccountInfo, MAX_PERMITTED_DATA_INCREASE},
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Start address of the memory region used for program heap.
pub const HEAP_START_ADDRESS: u64 = 0x300000000;

/// Length of the heap memory region used for program heap.
pub const HEAP_LENGTH: usize = 32 * 1024;

/// Maximum number of accounts that a transaction may process.
///
/// This value is used to set the maximum number of accounts that a program
/// is expecting and statically initialize the array of `AccountInfo`.
///
/// This is based on the current [maximum number of accounts] that a transaction
/// may lock in a block.
///
/// [maximum number of accounts]: https://github.com/anza-xyz/agave/blob/2e6ca8c1f62db62c1db7f19c9962d4db43d0d550/runtime/src/bank.rs#L3209-L3221
pub const MAX_TX_ACCOUNTS: usize = 128;

/// `assert_eq(std::mem::align_of::<u128>(), 8)` is true for BPF but not
/// for some host machines.
pub const BPF_ALIGN_OF_U128: usize = 8;

/// Value used to indicate that a serialized account is not a duplicate.
pub const NON_DUP_MARKER: u8 = u8::MAX;

/// Return value for a successful program execution.
pub const SUCCESS: u64 = 0;

/// The result of a program execution.
pub type ProgramResult = Result<(), ProgramError>;

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
///         entrypoint::ProgramResult,
///         msg,
///         pubkey::Pubkey
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
///
///         Ok(())
///     }
///
/// }
/// ```
#[macro_export]
macro_rules! entrypoint {
    ( $process_instruction:ident ) => {
        entrypoint!($process_instruction, {
            $crate::entrypoint::MAX_TX_ACCOUNTS
        });
    };
    ( $process_instruction:ident, $maximum:expr ) => {
        #[no_mangle]
        pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
            const UNINIT: std::mem::MaybeUninit<$crate::account_info::AccountInfo> =
                std::mem::MaybeUninit::<$crate::account_info::AccountInfo>::uninit();
            // create an array of uninitialized account infos; it is safe to `assume_init` since
            // we are claiming that the array of `MaybeUninit` is initialized and `MaybeUninit` do
            // not require initialization
            let mut accounts = [UNINIT; $maximum];

            let (program_id, count, instruction_data) =
                $crate::entrypoint::deserialize::<$maximum>(input, &mut accounts);

            // call the program's entrypoint passing `count` account infos; we know that
            // they are initialized so we cast the pointer to a slice of `[AccountInfo]`
            match $process_instruction(
                &program_id,
                std::slice::from_raw_parts(accounts.as_ptr() as _, count),
                &instruction_data,
            ) {
                Ok(()) => $crate::entrypoint::SUCCESS,
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
    accounts: &mut [std::mem::MaybeUninit<AccountInfo>],
) -> (&'a Pubkey, usize, &'a [u8]) {
    let mut offset: usize = 0;

    // total number of accounts present; it only process up to MAX_ACCOUNTS
    let total_accounts = *(input.add(offset) as *const u64) as usize;

    // number of processed accounts
    let count = if total_accounts <= MAX_ACCOUNTS {
        total_accounts
    } else {
        MAX_ACCOUNTS
    };

    offset += std::mem::size_of::<u64>();

    for i in 0..count {
        let duplicate_info = *(input.add(offset) as *const u8);
        if duplicate_info == NON_DUP_MARKER {
            let account_info: *mut Account = input.add(offset) as *mut _;

            offset += std::mem::size_of::<Account>();
            offset += (*account_info).data_len as usize;
            offset += MAX_PERMITTED_DATA_INCREASE;
            offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128);
            offset += std::mem::size_of::<u64>(); // MAGNETAR FIELDS: ignore rent epoch

            // MAGNETAR FIELDS: reset borrow state right before pushing
            (*account_info).borrow_state = 0b_0000_0000;

            accounts[i].write(AccountInfo {
                raw: account_info as *const _ as *mut _,
            });
        } else {
            offset += 8;
            // duplicate account, clone the original pointer
            accounts[i].write(accounts[duplicate_info as usize].assume_init_ref().clone());
        }
    }

    // process any remaining accounts to move the offset to the instruction
    // data (there is a duplication of logic but we avoid testing whether we
    // have space for the account or not)
    for _ in count..total_accounts {
        let duplicate_info = *(input.add(offset) as *const u8);

        if duplicate_info == NON_DUP_MARKER {
            let account_info: *mut Account = input.add(offset) as *mut _;
            offset += std::mem::size_of::<Account>();
            offset += (*account_info).data_len as usize;
            offset += MAX_PERMITTED_DATA_INCREASE;
            offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128);
            offset += std::mem::size_of::<u64>(); // MAGNETAR FIELDS: ignore rent epoch
        } else {
            offset += 8;
        }
    }

    // instruction data
    #[allow(clippy::cast_ptr_alignment)]
    let instruction_data_len = *(input.add(offset) as *const u64) as usize;
    offset += std::mem::size_of::<u64>();

    let instruction_data = { from_raw_parts(input.add(offset), instruction_data_len) };
    offset += instruction_data_len;

    // program id
    let program_id: &Pubkey = &*(input.add(offset) as *const Pubkey);

    (program_id, count, instruction_data)
}

#[macro_export]
macro_rules! custom_panic_default {
    () => {
        #[cfg(all(not(feature = "custom-panic"), target_os = "solana"))]
        #[no_mangle]
        fn custom_panic(info: &core::panic::PanicInfo<'_>) {
            // Full panic reporting
            $crate::msg!("{}", info);
        }
    };
}

#[macro_export]
macro_rules! custom_heap_default {
    () => {
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
unsafe impl std::alloc::GlobalAlloc for BumpAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pos_ptr = self.start as *mut usize;

        let mut pos = *pos_ptr;
        if pos == 0 {
            // First time, set starting position
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
