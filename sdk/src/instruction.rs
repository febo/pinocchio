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
