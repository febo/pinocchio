use core::{
    mem::{transmute, MaybeUninit},
    slice::from_raw_parts,
};

use pinocchio::{
    account_info::{AccountInfo, Ref},
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{write_bytes, ID, UNINIT_BYTE};

pub const MAX_ACCOUNTS_FOR_WITHDRAW: usize = 10;

/// State

#[repr(C)]
pub struct TransferFee {
    /// First epoch where the transfer fee takes effect
    pub epoch: [u8; 8],
    /// Maximum fee assessed on transfers, expressed as an amount of tokens
    pub maximum_fee: [u8; 8],
    /// Amount of transfer collected as fees, expressed as basis points of the
    /// transfer amount, ie. increments of 0.01%
    pub transfer_fee_basis_points: [u8; 8],
}

#[repr(C)]
pub struct TransferFeeConfig {
    /// flag to indicate if the transfer fee config authority is present
    pub transfer_fee_config_authority_flag: [u8; 4],
    /// Optional authority to set the fee
    pub transfer_fee_config_authority: Pubkey,
    /// flag to indicate if the withdraw authority is present
    pub withdraw_withheld_authority_flag: [u8; 4],
    /// Withdraw from mint instructions must be signed by this key
    pub withdraw_withheld_authority: Pubkey,
    /// Withheld transfer fee tokens that have been moved to the mint for
    /// withdrawal
    pub withheld_amount: [u8; 8],
    /// Older transfer fee, used if the current epoch < new_transfer_fee.epoch
    pub older_transfer_fee: TransferFee,
    /// Newer transfer fee, used if the current epoch >= new_transfer_fee.epoch
    pub newer_transfer_fee: TransferFee,
}

impl TransferFeeConfig {
    /// The length of the `TransferFeeConfig` account data.
    pub const LEN: usize = core::mem::size_of::<TransferFeeConfig>();

    /// Return a `TransferFeeConfig` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, safe borrowing
    /// the account data.
    #[inline(always)]
    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<Ref<TransferFeeConfig>, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner() != &ID {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(Ref::map(account_info.try_borrow_data()?, |data| unsafe {
            Self::from_bytes(data)
        }))
    }

    /// Return a `TransferFeeConfig` from the given account info.
    ///
    /// This method performs owner and length validation on `AccountInfo`, but does not
    /// perform the borrow check.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it is safe to borrow the account data – e.g., there are
    /// no mutable borrows of the account data.
    #[inline]
    pub unsafe fn from_account_info_unchecked(
        account_info: &AccountInfo,
    ) -> Result<&Self, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if account_info.owner() != &ID {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(Self::from_bytes(account_info.borrow_data_unchecked()))
    }

    /// Return a `TransferFeeConfig` from the given bytes.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` contains a valid representation of `TransferFeeConfig`.
    #[inline(always)]
    pub unsafe fn from_bytes(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const TransferFeeConfig)
    }
}

/// Instruction

pub struct InitializeTransferFeeConfig<'a> {
    // Mint account
    pub mint: &'a AccountInfo,
    /// Pubkey that may update the fees
    pub transfer_fee_config_authority: Option<Pubkey>,
    /// Withdraw instructions must be signed by this key
    pub withdraw_withheld_authority: Option<Pubkey>,
    /// Amount of transfer collected as fees, expressed as basis points of
    /// the transfer amount
    pub transfer_fee_basis_points: u16,
    /// Maximum fee assessed on transfers
    pub maximum_fee: u64,
}

impl<'a> InitializeTransferFeeConfig<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..33]: mint (32 bytes, Pubkey)
        // -  [33]: transfer_fee_config_authority_flag (1 byte, u8)
        // -  [34..66]: transfer_fee_config_authority (32 bytes, Pubkey)
        // -  [66]: withdraw_withheld_authority_flag (1 byte, u8)
        // -  [67..99]: withdraw_withheld_authority (32 bytes, Pubkey)
        // -  [99..101]: transfer_fee_basis_points (2 bytes, u16)
        // -  [101..109]: maximum_fee (8 bytes, u64)

        let mut instruction_data = [UNINIT_BYTE; 109];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[27]);
        // Set mint as Pubkey at offset [1..33]
        write_bytes(&mut instruction_data[1..33], self.mint.key().as_ref());
        // Set transfer_fee_config_authority COption at offset [33..37]
        let mut offset = 33;
        if let Some(transfer_fee_config_authority) = self.transfer_fee_config_authority {
            write_bytes(&mut instruction_data[33..34], &[1]);
            write_bytes(
                &mut instruction_data[34..66],
                transfer_fee_config_authority.as_ref(),
            );
            offset += 33;
        } else {
            write_bytes(&mut instruction_data[33..34], &[0]);
            offset += 1;
        }

        if let Some(withdraw_withheld_authority) = self.withdraw_withheld_authority {
            write_bytes(&mut instruction_data[offset..offset + 1], &[1]);
            write_bytes(
                &mut instruction_data[(offset + 1)..(offset + 1 + 32)],
                withdraw_withheld_authority.as_ref(),
            );
        } else {
            write_bytes(&mut instruction_data[offset..offset + 33], &[0]);
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &[AccountMeta::writable(self.mint.key())],
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 109) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}

pub struct TransferCheckedWithFee<'a> {
    /// Source account
    pub source: &'a AccountInfo,
    /// Token mint
    pub mint: &'a AccountInfo,
    /// Destination account
    pub destination: &'a AccountInfo,
    /// Transfer authority (owner or delegate)
    pub authority: &'a AccountInfo,
    /// The amount of tokens to transfer.
    pub amount: u64,
    /// Expected number of base 10 digits to the right of the decimal place.
    pub decimals: u8,
    /// Expected fee assessed on this transfer, calculated off-chain based
    /// on the transfer_fee_basis_points and maximum_fee of the mint. May
    /// be 0 for a mint without a configured transfer fee.
    pub fee: u64,
}

impl<'a> TransferCheckedWithFee<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 4] = [
            AccountMeta::writable(self.source.key()),
            AccountMeta::writable(self.mint.key()),
            AccountMeta::writable(self.destination.key()),
            AccountMeta::readonly_signer(self.authority.key()),
        ];

        // Instruction data layout:
        // -  [0]: instruction discriminator (1 byte, u8)
        // -  [1..9]: amount (8 bytes, u64)
        // -  [9]: decimals (1 byte, u8)
        // -  [10..18]: fee (8 bytes, u64)
        let mut instruction_data = [UNINIT_BYTE; 18];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[28]);
        // Set amount as u64 at offset [1..9]
        write_bytes(&mut instruction_data[1..9], &self.amount.to_le_bytes());
        // Set decimals as u8 at offset [9]
        write_bytes(&mut instruction_data[9..10], &[self.decimals]);
        // Set fee as u64 at offset [10..18]
        write_bytes(&mut instruction_data[10..18], &self.fee.to_le_bytes());

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 18) },
        };

        invoke_signed(
            &instruction,
            &[self.source, self.mint, self.destination, self.authority],
            signers,
        )
    }
}

pub struct WithdrawWithheldTokensFromMint<'a> {
    /// Mint account (must include the `TransferFeeConfig` extension)
    pub mint: &'a AccountInfo,
    /// The fee receiver account (must include the `TransferFeeAmount` extension associated with the provided mint)
    pub fee_receiver: &'a AccountInfo,
    /// The mint's `withdraw_withheld_authority`.
    pub withraw_withheld_authority: &'a AccountInfo,
}

impl<'a> WithdrawWithheldTokensFromMint<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        let account_metas: [AccountMeta; 3] = [
            AccountMeta::writable(self.mint.key()),
            AccountMeta::writable(self.fee_receiver.key()),
            AccountMeta::readonly_signer(self.withraw_withheld_authority.key()),
        ];

        // Instruction data layout:
        // -  [0]: instruction discriminator
        let instruction_data = [29];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        invoke_signed(
            &instruction,
            &[
                self.mint,
                self.fee_receiver,
                self.withraw_withheld_authority,
            ],
            signers,
        )
    }
}

pub struct WithdrawWithheldTokensFromAccounts<'a> {
    /// Mint account (must include the `TransferFeeConfig` extension)
    pub mint: &'a AccountInfo,
    /// The fee receiver account (must include the `TransferFeeAmount` extension associated with the provided mint)
    pub fee_receiver: &'a AccountInfo,
    /// The mint's `withdraw_withheld_authority`.
    pub withdraw_withheld_authority: &'a AccountInfo,
    /// The source accounts to withdraw from.
    pub source_accounts: &'a [AccountInfo],
}

impl<'a> WithdrawWithheldTokensFromAccounts<'a> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        // Account metadata
        // let mut account_metas =
        //     unsafe { MaybeUninit::<[AccountMeta; MAX_ACCOUNTS_FOR_WITHDRAW + 3]>::uninit() };

        let account_metas = {
            let mut data = [const { MaybeUninit::uninit() }; MAX_ACCOUNTS_FOR_WITHDRAW + 3];

            data[0] = MaybeUninit::new(AccountMeta::writable(self.mint.key()));

            data[1] = MaybeUninit::new(AccountMeta::writable(self.fee_receiver.key()));
            data[2] = MaybeUninit::new(AccountMeta::readonly_signer(
                self.withdraw_withheld_authority.key(),
            ));

            for (i, account) in self.source_accounts.iter().enumerate() {
                data[i] = MaybeUninit::new(AccountMeta::writable(account.key()));
            }

            unsafe { transmute::<_, [AccountMeta; MAX_ACCOUNTS_FOR_WITHDRAW + 3]>(data) }
        };
        // Instruction data layout:
        // -  [0]: instruction discriminator
        let instruction_data = [30];

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &account_metas,
            data: &instruction_data,
        };

        let accounts = {
            let mut accounts = [MaybeUninit::uninit(); MAX_ACCOUNTS_FOR_WITHDRAW + 3];

            accounts[0] = MaybeUninit::new(self.mint);
            accounts[1] = MaybeUninit::new(self.fee_receiver);
            accounts[2] = MaybeUninit::new(self.withdraw_withheld_authority);

            for (i, account) in self.source_accounts.iter().enumerate() {
                accounts[3 + i] = MaybeUninit::new(account);
            }

            unsafe { transmute::<_, [&AccountInfo; MAX_ACCOUNTS_FOR_WITHDRAW + 3]>(accounts) }
        };

        invoke_signed(&instruction, &accounts, signers)
    }
}
