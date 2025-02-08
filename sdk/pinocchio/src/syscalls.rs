//! Syscall functions.

#[cfg(target_feature = "static-syscalls")]
pub use solana_define_syscall::sys_hash;
pub use solana_define_syscall::{define_syscall, definitions::*};
