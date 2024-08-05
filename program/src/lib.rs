//! A library to build a Solana Program in Rust.
//!
//! This library is intended to be used by on-chain programs only. It provides
//! a zero-dependency library to minimise dependencies conflits. For off-chain
//! programs, use instead the [`solana-sdk`] crate, which reexports all modules
//! from [`solana-program`].
//!
//! [`solana-sdk`]: https://docs.rs/solana-sdk/latest/solana_sdk/
//! [`solana-program`]: https://docs.rs/solana-program/latest/solana_program/

pub mod account_info;
pub mod entrypoint;
pub mod instruction;
pub mod log;
pub mod program_error;
pub mod pubkey;
pub mod syscalls;

/// Maximum number of bytes a program may add to an account during a
/// single realloc.
pub const MAX_PERMITTED_DATA_INCREASE: usize = 1_024 * 10;

/// `assert_eq(std::mem::align_of::<u128>(), 8)` is true for BPF but not
/// for some host machines./
pub const BPF_ALIGN_OF_U128: usize = 8;

/// Value used to indicate that a serialized account is not a duplicate.
pub const NON_DUP_MARKER: u8 = u8::MAX;
