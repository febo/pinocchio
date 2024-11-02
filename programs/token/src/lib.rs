#![no_std]

use core::mem::MaybeUninit;
pub mod instructions;
pub mod state;

const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();

pinocchio_pubkey::declare_id!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

#[inline(always)]
fn write_bytes(destination: &mut [MaybeUninit<u8>], source: &[u8]) {
    for (d, s) in destination.iter_mut().zip(source.iter()) {
        d.write(*s);
    }
}

