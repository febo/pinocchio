use crate::pubkey::Pubkey;

/// An `AccountMeta`` as expected by `sol_invoke_signed_c`.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct AccountMeta {
    // Public key of the account.
    pubkey: *const Pubkey,

    // Is the account writable?
    pub is_writable: bool,

    // Transaction was signed by this account's key?
    pub is_signer: bool,
}

impl AccountMeta {
    #[inline(always)]
    pub fn pubkey(&self) -> &Pubkey {
        unsafe { &*self.pubkey }
    }
}

impl From<&crate::account_info::AccountInfo> for AccountMeta {
    fn from(account: &crate::account_info::AccountInfo) -> Self {
        AccountMeta {
            pubkey: offset(account.raw, 8),
            is_writable: account.is_writable(),
            is_signer: account.is_signer(),
        }
    }
}

/// An `AccountInfo`` as expected by `sol_invoke_signed_c`.
#[repr(C)]
#[derive(Clone)]
pub struct AccountInfo {
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

impl From<&crate::account_info::AccountInfo> for AccountInfo {
    fn from(account: &crate::account_info::AccountInfo) -> Self {
        AccountInfo {
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
#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct Instruction {
    /// Public key of the program.
    pub program_id: *const Pubkey,

    /// Accounts expected by the program instruction.
    pub accounts: *const AccountMeta,

    /// Number of accounts expected by the program instruction.
    pub accounts_len: u64,

    /// Data expected by the program instruction.
    pub data: *const u8,

    /// Length of the data expected by the program instruction.
    pub data_len: u64,
}

/// A signer seed as expected by `sol_invoke_signed_c`.
#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct SignerSeed {
    /// Seed bytes.
    pub seed: *const u8,

    /// Length of the seed bytes.
    pub len: u64,
}

/// Signer as expected by `sol_invoke_signed_c`.
#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct Signer {
    /// Seed bytes.
    pub seeds: *const SignerSeed,

    /// Number of signers.
    pub len: u64,
}

/// Use to query and convey information about the sibling instruction components
/// when calling the `sol_get_processed_sibling_instruction` syscall.
#[repr(C)]
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct ProcessedSiblingInstruction {
    /// Length of the instruction data
    pub data_len: u64,

    /// Number of AccountMeta structures
    pub accounts_len: u64,
}
