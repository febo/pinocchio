pub use five8_const::decode_32_const;
pub use pinocchio;

#[macro_export]
macro_rules! declare_pubkey {
    ( $id:expr ) => {
        $crate::decode($id)
    };
}

#[macro_export]
macro_rules! declare_id {
    ( $id:expr ) => {
        #[doc = "The const program ID."]
        pub const ID: $crate::pinocchio::pubkey::Pubkey = $crate::decode($id);

        #[doc = "Returns `true` if given pubkey is the program ID."]
        #[inline]
        pub fn check_id(id: &$crate::pinocchio::pubkey::Pubkey) -> bool {
            id == &ID
        }

        #[doc = "Returns the program ID."]
        #[inline]
        pub const fn id() -> $crate::pinocchio::pubkey::Pubkey {
            ID
        }
    };
}

#[inline(always)]
pub const fn decode(value: &str) -> pinocchio::pubkey::Pubkey {
    decode_32_const(value)
}
