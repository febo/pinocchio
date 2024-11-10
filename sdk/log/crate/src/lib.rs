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
//! Creating a `Logger` with a buffer size of 100 bytes, and appending a string and an
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

#![no_std]

pub mod logger;

#[cfg(feature = "macro")]
pub use pinocchio_log_macro::*;

#[cfg(test)]
mod tests {
    use crate::logger::Logger;

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
}
