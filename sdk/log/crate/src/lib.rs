#![no_std]

pub use pinocchio_log_macro::*;

pub mod logger;

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
}
