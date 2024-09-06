use crate::{account_info::AccountInfo, pubkey::Pubkey};

/// Information about a CPI instruction.
#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct Instruction<'a, 'b> {
    /// Public key of the program.
    pub program_id: &'a Pubkey,

    /// Data expected by the program instruction.
    pub data: &'b [u8],
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

/// Describes a single account read or written by a program during instruction
/// execution.
///
/// When constructing an [`Instruction`], a list of all accounts that may be
/// read or written during the execution of that instruction must be supplied.
/// Any account that may be mutated by the program during execution, either its
/// data or metadata such as held lamports, must be writable.
///
/// Note that because the Solana runtime schedules parallel transaction
/// execution around which accounts are writable, care should be taken that only
/// accounts which actually may be mutated are specified as writable.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct AccountMeta<'a> {
    // Public key of the account.
    pub pubkey: &'a Pubkey,

    // Is the account writable?
    pub is_writable: bool,

    // Transaction was signed by this account's key?
    pub is_signer: bool,
}

impl<'a> AccountMeta<'a> {
    pub fn new(pubkey: &'a Pubkey, is_writable: bool, is_signer: bool) -> Self {
        Self {
            pubkey,
            is_writable,
            is_signer,
        }
    }

    #[inline(always)]
    pub fn readonly(pubkey: &'a Pubkey) -> Self {
        Self::new(pubkey, false, false)
    }

    #[inline(always)]
    pub fn writable(pubkey: &'a Pubkey) -> Self {
        Self::new(pubkey, true, false)
    }

    #[inline(always)]
    pub fn readonly_signer(pubkey: &'a Pubkey) -> Self {
        Self::new(pubkey, false, true)
    }

    #[inline(always)]
    pub fn writable_signer(pubkey: &'a Pubkey) -> Self {
        Self::new(pubkey, true, true)
    }
}

impl<'a> From<&'a AccountInfo> for AccountMeta<'a> {
    fn from(account: &'a crate::account_info::AccountInfo) -> Self {
        AccountMeta {
            pubkey: account.key(),
            is_writable: account.is_writable(),
            is_signer: account.is_signer(),
        }
    }
}
