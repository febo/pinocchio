#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccountState {
    Uninitialized,
    Initialized,
    Frozen,
}
