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

pub fn find_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
    let mut bytes = [0; 32];
    let mut bump_seed = u8::MAX;
    let result = unsafe {
        crate::syscalls::sol_try_find_program_address(
            seeds as *const _ as *const u8,
            seeds.len() as u64,
            program_id as *const _ as *const u8,
            &mut bytes as *mut _ as *mut u8,
            &mut bump_seed as *mut _,
        )
    };
    match result {
        crate::entrypoint::SUCCESS => (bytes, bump_seed),
        _ => panic!("PDA cannot be found"),
    }
}
