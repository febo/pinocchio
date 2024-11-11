use core::slice::from_raw_parts;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{write_bytes, UNINIT_BYTE};

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
        // -  [0]: instruction discriminator
        // -  [1..33]: mint
        // -  [33]: transfer_fee_config_authority_flag
        // -  [34..66]: transfer_fee_config_authority
        // -  [66]: withdraw_withheld_authority_flag
        // -  [67..99]: withdraw_withheld_authority
        // -  [99..101]: transfer_fee_basis_points
        // -  [101..109]: maximum_fee

        let mut instruction_data = [UNINIT_BYTE; 83];

        // Set discriminator as u8 at offset [0]
        write_bytes(&mut instruction_data, &[todo!()]);
        // Set mint as Pubkey at offset [1..33]
        write_bytes(&mut instruction_data[1..33], self.mint.key().as_ref());
        // Set transfer_fee_config_authority COption at offset [33..37]
        if let Some(transfer_fee_config_authority) = self.transfer_fee_config_authority {
            write_bytes(&mut instruction_data[33..37], &[1]);
            write_bytes(
                &mut instruction_data[37..69],
                transfer_fee_config_authority.as_ref(),
            );
        } else {
            write_bytes(&mut instruction_data[33..37], &[0]);
        }

        let instruction = Instruction {
            program_id: &crate::ID,
            accounts: &[AccountMeta::writable(self.mint.key())],
            data: unsafe { from_raw_parts(instruction_data.as_ptr() as _, 9) },
        };

        invoke_signed(&instruction, &[self.mint], signers)
    }
}
