/// Declare the program entrypoint with a global allocator that prevents heap allocations and a
/// default panic handler.
///
/// The main difference from the `entrypoint!` macro is that this macro sets up a global allocator
/// that panics when an allocation is attempted. This is useful when the program does not need to
/// allocate memory. Other aspects of the entrypoint are the same as the `entrypoint!` macro.
///
/// Note that it is not possible to use this macro with the "`std`" feature enabled.
///
/// # Example
///
/// Defining a `heapless_entrypoint` conditional on the `bpf-entrypoint` feature.
///
/// ```no_run
/// #[cfg(feature = "bpf-entrypoint")]
/// pub mod entrypoint {
///
///     use pinocchio::{
///         account_info::AccountInfo,
///         heapless_entrypoint,
///         msg,
///         pubkey::Pubkey,
///         ProgramResult
///     };
///
///     heapless_entrypoint!(process_instruction);
///
///     pub fn process_instruction(
///         program_id: &Pubkey,
///         accounts: &[AccountInfo],
///         instruction_data: &[u8],
///     ) -> ProgramResult {
///         msg!("Hello from my `heapless` program!");
///         Ok(())
///     }
///
/// }
/// ```
#[macro_export]
macro_rules! heapless_entrypoint {
    ( $process_instruction:ident ) => {
        heapless_entrypoint!($process_instruction, { $crate::MAX_TX_ACCOUNTS });
    };
    ( $process_instruction:ident, $maximum:expr ) => {
        $crate::program_entrypoint!($process_instruction, $maximum);
        $crate::heapless_allocator!();
        $crate::default_panic_handler!();
    };
}

/// A global allocator that does not allocate heap memory.
///
/// Using this macro with the "`std`" feature enabled will result in a compile error.
#[cfg(feature = "std")]
#[macro_export]
macro_rules! heapless_allocator {
    () => {
        compile_error!("Feature 'std' cannot be enabled.");
    };
}

/// Zero global allocator.
///
/// This macro sets up a global allocator that denies all allocations. This is useful when the
/// program does not need to allocate memory.
#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! heapless_allocator {
    () => {
        #[cfg(target_os = "solana")]
        #[global_allocator]
        static A: $crate::entrypoint::heapless::HeaplessAllocator =
            $crate::entrypoint::heapless::HeaplessAllocator;
    };
}

#[cfg(not(feature = "std"))]
/// Zero global allocator.
pub struct HeaplessAllocator;

#[cfg(not(feature = "std"))]
unsafe impl core::alloc::GlobalAlloc for HeaplessAllocator {
    #[inline]
    unsafe fn alloc(&self, _: core::alloc::Layout) -> *mut u8 {
        panic!("** HEAPLESS ALLOCATOR **");
    }

    #[inline]
    unsafe fn dealloc(&self, _: *mut u8, _: core::alloc::Layout) {
        // I deny all allocations, so I don't need to free.
    }
}
