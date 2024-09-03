/// The address of a [Solana account][account].
///
/// [account]: https://solana.com/docs/core/accounts
pub type Pubkey = [u8; 32];

/// Log a `Pubkey` from a program
pub fn log_pubkey(pubkey: &Pubkey) {
    #[cfg(target_os = "solana")]
    unsafe {
        crate::syscalls::sol_log_pubkey(pubkey as *const _ as *const u8)
    };

    #[cfg(not(target_os = "solana"))]
    core::hint::black_box(pubkey);
}
