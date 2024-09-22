pub mod pubkey;

#[macro_export]
macro_rules! declare_pubkey {
    ( $id:expr ) => {
        pinocchio_macro::pubkey::decode($id)
    };
}

#[macro_export]
macro_rules! declare_id {
    ( $id:expr ) => {
        #[doc = "The const program ID."]
        pub const ID: pinocchio::pubkey::Pubkey = pinocchio_macro::pubkey::decode($id);

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
