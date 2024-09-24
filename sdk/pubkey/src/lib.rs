pub use five8_const::decode_32_const;
use pinocchio::pubkey::Pubkey;

#[macro_export]
macro_rules! declare_pubkey {
    ( $id:expr ) => {
        pinocchio_pubkey::decode($id)
    };
}

#[macro_export]
macro_rules! declare_id {
    ( $id:expr ) => {
        #[doc = "The const program ID."]
        pub const ID: pinocchio::pubkey::Pubkey = pinocchio_pubkey::decode($id);

        #[doc = "Returns `true` if given pubkey is the program ID."]
        #[inline]
        pub fn check_id(id: &pinocchio::pubkey::Pubkey) -> bool {
            id == &ID
        }

        #[doc = "Returns the program ID."]
        #[inline]
        pub const fn id() -> pinocchio::pubkey::Pubkey {
            ID
        }
    };
}

#[inline(always)]
pub const fn decode(value: &str) -> Pubkey {
    decode_32_const(value)
}
