#[derive(Debug, Clone, Copy)]
pub enum AccountState {
    Uninitialized,
    Initialized,
    Frozen,
}