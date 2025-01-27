//! Adapter types to facilitate the interaction between SDK types and pinocchio.

#![no_std]

pub mod account_info;

/// Converts a `pinocchio::program_error::ProgramError` to a
/// `solana_program_error::ProgramError`.
#[macro_export]
macro_rules! to_program_error {
    ($error:expr) => {
        solana_program_error::ProgramError::from(u64::from($error))
    };
}
