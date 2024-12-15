pub mod confidential_transfer;
pub mod cpi_guard;
pub mod default_account_state;
pub mod memo_transfer;
pub mod transfer_fee;

pub const ELGAMAL_PUBKEY_LEN: usize = 32;

pub struct ElagamalPubkey(pub [u8; ELGAMAL_PUBKEY_LEN]);
