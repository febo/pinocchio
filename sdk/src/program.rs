use crate::{
    instruction::{Account, AccountMeta, Instruction, Signer},
    pubkey::Pubkey,
};

/// An `Instruction` as expected by `sol_invoke_signed_c`.
///
/// DO NOT EXPOSE THIS STRUCT:
///
/// To ensure pointers are valid upon use, the scope of this struct should
/// only be limited to the stack where sol_invoke_signed_c happens and then
/// discarded immediately after.
#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
struct CInstruction<'a> {
    /// Public key of the program.
    program_id: *const Pubkey,

    /// Accounts expected by the program instruction.
    accounts: *const AccountMeta<'a>,

    /// Number of accounts expected by the program instruction.
    accounts_len: u64,

    /// Data expected by the program instruction.
    data: *const u8,

    /// Length of the data expected by the program instruction.
    data_len: u64,
}

impl<'a> From<&Instruction<'a, '_, '_, '_>> for CInstruction<'a> {
    fn from(instruction: &Instruction<'a, '_, '_, '_>) -> Self {
        CInstruction {
            program_id: instruction.program_id,
            accounts: instruction.accounts.as_ptr(),
            accounts_len: instruction.accounts.len() as u64,
            data: instruction.data.as_ptr(),
            data_len: instruction.data.len() as u64,
        }
    }
}

/// Invoke a cross-program instruction with signatures but don't enforce Rust's
/// aliasing rules.
///
/// This function is like [`invoke_signed`] except that it does not check that
/// [`RefCell`]s within [`Account`]s are properly borrowable as described in
/// the documentation for that function. Those checks consume CPU cycles that
/// this function avoids.
///
/// [`RefCell`]: std::cell::RefCell
///
/// # Safety
///
/// If any of the writable accounts passed to the callee contain data that is
/// borrowed within the calling program, and that data is written to by the
/// callee, then Rust's aliasing rules will be violated and cause undefined
/// behavior.
pub unsafe fn invoke_signed_unchecked(
    instruction: &Instruction,
    accounts: &[Account],
    signers_seeds: &[Signer],
) {
    #[cfg(target_os = "solana")]
    {
        let instruction = CInstruction::from(instruction);
        unsafe {
            crate::syscalls::sol_invoke_signed_c(
                &instruction as *const _ as *const u8,
                accounts as *const _ as *const u8,
                accounts.len() as u64,
                signers_seeds as *const _ as *const u8,
                signers_seeds.len() as u64,
            )
        };
    }

    #[cfg(not(target_os = "solana"))]
    core::hint::black_box((instruction, accounts, signers_seeds));
}
