//! A library to build a Solana Program in Rust.
//!
//! This library is intended to be used by on-chain programs only. It provides
//! a zero-dependency library to minimise dependencies conflits. For off-chain
//! programs, use instead the [`solana-sdk`] crate, which reexports all modules
//! from [`solana-program`].
//!
//! [`solana-sdk`]: https://docs.rs/solana-sdk/latest/solana_sdk/
//! [`solana-program`]: https://docs.rs/solana-program/latest/solana_program/
//!

#![no_std]

extern crate alloc;

pub mod account_info;
pub mod entrypoint;
pub mod instruction;
pub mod log;
pub mod program;
pub mod program_error;
pub mod pubkey;
pub mod syscalls;
pub mod sysvars;

#[cfg(feature = "macro")]
pub use pinocchio_macro::{declare_id, pubkey};
