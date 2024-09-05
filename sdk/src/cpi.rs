// Reference: https://github.com/anza-xyz/agave/blob/e207c6e0eaf8e1657fbfaff07da05ca6a7928349/programs/sbf/rust/ro_modify/src/lib.rs#L105-L129

use crate::{
    account_info::AccountInfo, instruction::Instruction, program_error::ProgramError,
    syscalls::sol_invoke_signed_c,
};

/// The struct expected to be pointed to by `sol_invoke_signed_c()`'s first arg.
///
/// The u64 fields are raw pointers.
///
/// DO NOT EXPOSE THIS STRUCT -
/// to ensure pointers are valid upon use, the scope of this struct should
/// only be limited to the stack where sol_invoke_signed_c happens and then
/// discarded immediately after
#[derive(Debug)]
#[repr(C)]
struct SolInstruction {
    program_id_addr: u64,
    accounts_addr: u64,
    accounts_len: usize,
    data_addr: u64,
    data_len: usize,
}

// Reference: https://github.com/anza-xyz/agave/blob/c8685ce0e1bb9b26014f1024de2cd2b8c308cbde/programs/sbf/rust/ro_modify/src/lib.rs#L122-L132
impl<'p, 'a, 'd> From<Instruction<'p, 'a, 'd>> for SolInstruction {
    #[inline]
    fn from(
        Instruction {
            program_id,
            accounts,
            data,
        }: Instruction,
    ) -> Self {
        Self {
            program_id_addr: program_id.as_ptr() as u64,
            accounts_addr: accounts.as_ptr() as u64,
            accounts_len: accounts.len(),
            data_addr: data.as_ptr() as u64,
            data_len: data.len(),
        }
    }
}

/// The array elem of `sol_invoke_signed_c()`'s `account_infos_addr` arg.
///
/// The u64 fields are raw pointers.
///
/// DO NOT EXPOSE THIS STRUCT -
/// to ensure pointers are valid upon use, the scope of this struct should
/// only be limited to the stack where sol_invoke_signed_c happens and then
/// discarded immediately after
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct SolAccountInfo {
    key_addr: u64,
    lamports_addr: u64,
    data_len: u64,
    data_addr: u64,
    owner_addr: u64,
    rent_epoch: u64,
    is_signer: bool,
    is_writable: bool,
    executable: bool,
}

impl SolAccountInfo {
    const fn null() -> Self {
        Self {
            key_addr: 0,
            lamports_addr: 0,
            data_len: 0,
            data_addr: 0,
            owner_addr: 0,
            rent_epoch: 0,
            is_signer: false,
            is_writable: false,
            executable: false,
        }
    }
}

impl From<&AccountInfo> for SolAccountInfo {
    #[inline]
    fn from(value: &AccountInfo) -> Self {
        let (data_len, data_addr) = {
            let a = unsafe { value.unchecked_borrow_data() };
            let len = a.len();
            (len, a.as_ptr())
        };
        let lamports_addr = unsafe { value.unchecked_borrow_lamports() } as *const _;
        Self {
            key_addr: value.key().as_ptr() as u64,
            lamports_addr: lamports_addr as u64,
            data_len: data_len as u64,
            data_addr: data_addr as u64,
            owner_addr: value.owner().as_ptr() as u64,
            is_signer: value.is_signer(),
            is_writable: value.is_writable(),
            executable: value.executable(),
            rent_epoch: 0, // dont care
        }
    }
}

pub fn invoke_signed<'a, const MAX_ACCOUNTS_INCL_PROGRAM: usize>(
    ix: Instruction,
    program_account_info: &'a AccountInfo,
    account_infos: impl IntoIterator<Item = &'a AccountInfo>,
    signers: &[&[&[u8]]],
) -> Result<(), ProgramError> {
    let (sol_account_infos, sol_account_infos_len) = core::iter::once(program_account_info)
        .chain(account_infos)
        .enumerate()
        .try_fold(
            ([SolAccountInfo::null(); MAX_ACCOUNTS_INCL_PROGRAM], 0),
            |(mut arr, len), (i, curr)| {
                if i >= MAX_ACCOUNTS_INCL_PROGRAM {
                    return Err(ProgramError::InvalidArgument);
                }
                arr[i] = SolAccountInfo::from(curr);
                Ok((arr, len + 1))
            },
        )?;
    let sol_ix = SolInstruction::from(ix);
    let result = unsafe {
        sol_invoke_signed_c(
            &sol_ix as *const _ as *const _,
            &sol_account_infos as *const _ as *const _,
            sol_account_infos_len,
            signers as *const _ as *const _,
            signers.len() as u64,
        )
    };
    match result {
        crate::entrypoint::SUCCESS => Ok(()),
        _ => Err(result.into()),
    }
}
