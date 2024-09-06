use crate::{account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey};

/// An `AccountInfo` as expected by `sol_invoke_signed_c`.
///
/// DO NOT EXPOSE THIS STRUCT:
///
/// To ensure pointers are valid upon use, the scope of this struct should
/// only be limited to the stack where sol_invoke_signed_c happens and then
/// discarded immediately after.
#[repr(C)]
#[derive(Clone)]
struct CAccountInfo {
    // Public key of the account.
    pub key: *const Pubkey,

    // Number of lamports owned by this account.
    pub lamports: *const u64,

    // Length of data in bytes.
    pub data_len: u64,

    // On-chain data within this account.
    pub data: *const u8,

    // Program that owns this account.
    pub owner: *const Pubkey,

    // The epoch at which this account will next owe rent.
    pub rent_epoch: u64,

    // Transaction was signed by this account's key?
    pub is_signer: bool,

    // Is the account writable?
    pub is_writable: bool,

    // This account's data contains a loaded program (and is now read-only).
    pub executable: bool,
}

#[inline(always)]
const fn offset<T, U>(ptr: *const T, offset: usize) -> *const U {
    unsafe { (ptr as *const u8).add(offset) as *const U }
}

impl From<&AccountInfo> for CAccountInfo {
    fn from(account: &AccountInfo) -> Self {
        CAccountInfo {
            key: offset(account.raw, 8),
            lamports: offset(account.raw, 72),
            data_len: account.data_len() as u64,
            data: offset(account.raw, 88),
            owner: offset(account.raw, 40),
            rent_epoch: 0,
            is_signer: account.is_signer(),
            is_writable: account.is_writable(),
            executable: account.executable(),
        }
    }
}

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
    pub program_id: *const Pubkey,

    /// Accounts expected by the program instruction.
    pub accounts: *const AccountMeta<'a>,

    /// Number of accounts expected by the program instruction.
    pub accounts_len: u64,

    /// Data expected by the program instruction.
    pub data: *const u8,

    /// Length of the data expected by the program instruction.
    pub data_len: u64,
}

/// A signer seed as expected by `sol_invoke_signed_c`.
///
/// DO NOT EXPOSE THIS STRUCT:
///
/// To ensure pointers are valid upon use, the scope of this struct should
/// only be limited to the stack where sol_invoke_signed_c happens and then
/// discarded immediately after.
#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
struct CSignerSeed {
    /// Seed bytes.
    pub seed: *const u8,

    /// Length of the seed bytes.
    pub len: u64,
}

/// Signer as expected by `sol_invoke_signed_c`.
///
/// DO NOT EXPOSE THIS STRUCT:
///
/// To ensure pointers are valid upon use, the scope of this struct should
/// only be limited to the stack where sol_invoke_signed_c happens and then
/// discarded immediately after.
#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
struct CSigner {
    /// Seed bytes.
    pub seeds: *const CSignerSeed,

    /// Number of signers.
    pub len: u64,
}
