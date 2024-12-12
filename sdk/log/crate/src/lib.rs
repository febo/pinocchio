//! Lightweight log utility for Solana programs.
//!
//! This crate provides a `Logger` struct that can be used to efficiently log messages
//! in a Solana program. The `Logger` struct is a wrapper around a fixed-size buffer,
//! where types that implement the `Log` trait can be appended to the buffer.
//!
//! The `Logger` struct is generic over the size of the buffer, and the buffer size
//! should be chosen based on the expected size of the log messages. When the buffer is
//! full, the log message will be truncated. This is represented by the `@` character
//! at the end of the log message.
//!
//! # Example
//!
//! Creating a `Logger` with a buffer size of `100` bytes, and appending a string and an
//! `u64` value:
//!
//! ```
//! use pinocchio_log::logger::Logger;
//!
//! let mut logger = Logger::<100>::default();
//! logger.append("balance=");
//! logger.append(1_000_000_000);
//! logger.log();
//!
//! // Clear the logger buffer.
//! logger.clear();
//!
//! logger.append(&["Hello ", "world!"]);
//! logger.log();
//! ```
//!
//! It also support adding precision to numeric types:
//!
//! ```
//! use pinocchio_log::logger::{Argument, Logger};
//!
//! let mut logger = Logger::<100>::default();
//!
//! let lamports = 1_000_000_000u64;
//!
//! logger.append("balance (SOL)=");
//! logger.append_with_args(lamports, &[Argument::Precision(9)]);
//! logger.log();
//! ```

#![no_std]

pub mod logger;

#[cfg(feature = "macro")]
pub use pinocchio_log_macro::*;

#[cfg(test)]
mod tests {
    use crate::logger::{Argument, Logger};

    #[test]
    fn test_logger() {
        let mut logger = Logger::<100>::default();
        logger.append("Hello ");
        logger.append("world!");

        assert!(&*logger == "Hello world!".as_bytes());

        logger.clear();

        logger.append("balance=");
        logger.append(1_000_000_000);

        assert!(&*logger == "balance=1000000000".as_bytes());
    }

    #[test]
    fn test_logger_trucated() {
        let mut logger = Logger::<8>::default();
        logger.append("Hello ");
        logger.append("world!");

        assert!(&*logger == "Hello w@".as_bytes());

        let mut logger = Logger::<12>::default();

        logger.append("balance=");
        logger.append(1_000_000_000);

        assert!(&*logger == "balance=100@".as_bytes());
    }

    #[test]
    fn test_logger_slice() {
        let mut logger = Logger::<20>::default();
        logger.append(&["Hello ", "world!"]);

        assert!(&*logger == "[\"Hello \", \"world!\"]".as_bytes());

        let mut logger = Logger::<20>::default();
        logger.append(&[123, 456]);

        assert!(&*logger == "[123, 456]".as_bytes());
    }

    #[test]
    fn test_logger_truncated_slice() {
        let mut logger = Logger::<5>::default();
        logger.append(&["Hello ", "world!"]);

        assert!(&*logger == "[\"He@".as_bytes());

        let mut logger = Logger::<4>::default();
        logger.append(&[123, 456]);

        assert!(&*logger == "[12@".as_bytes());
    }

    #[test]
    fn test_logger_signed() {
        let mut logger = Logger::<2>::default();
        logger.append(-2);

        assert!(&*logger == "-2".as_bytes());

        let mut logger = Logger::<5>::default();
        logger.append(-200_000_000);

        assert!(&*logger == "-200@".as_bytes());
    }

    #[test]
    fn test_logger_with_precision() {
        let mut logger = Logger::<10>::default();

        logger.append_with_args(200_000_000u64, &[Argument::Precision(2)]);
        assert!(&*logger == "2000000.00".as_bytes());

        logger.clear();

        logger.append_with_args(2_000_000_000u64, &[Argument::Precision(2)]);
        assert!(&*logger == "20000000.@".as_bytes());

        logger.clear();

        logger.append_with_args(2_000_000_000u64, &[Argument::Precision(5)]);
        assert!(&*logger == "20000.000@".as_bytes());

        logger.clear();

        logger.append_with_args(2_000_000_000u64, &[Argument::Precision(10)]);
        assert!(&*logger == "0.2000000@".as_bytes());

        logger.clear();

        logger.append_with_args(2u64, &[Argument::Precision(6)]);
        assert!(&*logger == "0.000002".as_bytes());

        logger.clear();

        logger.append_with_args(2u64, &[Argument::Precision(9)]);
        assert!(&*logger == "0.0000000@".as_bytes());

        logger.clear();

        logger.append_with_args(-2000000i32, &[Argument::Precision(6)]);
        assert!(&*logger == "-2.000000".as_bytes());

        logger.clear();

        logger.append_with_args(-2i64, &[Argument::Precision(9)]);
        assert!(&*logger == "-0.000000@".as_bytes());

        logger.clear();

        logger.append_with_args("0123456789", &[Argument::Precision(10)]);
        assert!(&*logger == "0123456789".as_bytes());

        logger.clear();

        logger.append_with_args("0123456789", &[Argument::Precision(9)]);
        assert!(&*logger == "...456789".as_bytes());

        let mut logger = Logger::<3>::default();
        logger.append_with_args("0123456789", &[Argument::Precision(9)]);
        assert!(&*logger == "..@".as_bytes());
    }
}
