# `system`

[`pinocchio`](https://crates.io/crates/pinocchio) helpers to perform cross-program invocations (CPIs) for System program instructions.

Each instruction defines an `struct` with the accounts and parameters required. Once all values are set, you can call directly `invoke` or `invoke_signed` to perform the CPI.

> [!IMPORTANT]
> The API defined in this crate should be considered experimental.

## Examples

Creating a new account:
```rust
// This example assumes that the instruction receives a writable signer `payer_info`
// and `new_account` accounts.
CreateAccount {
    from: payer_info,
    to: new_account,
    lamports: 1_000_000_000, // 1 SOL
    space: 200,              // 200 bytes
    owner: &spl_token::ID,
}.invoke()?;
```

Performing a transfer of lamports:
```rust
// This example assumes that the instruction receives a writable signer `payer_info`
// account and a writable `recipient` account.
Transfer {
    from: payer_info,
    to: recipient,
    lamports: 500_000_000, // 0.5 SOL
}.invoke()?;
```

## License

The code is licensed under the [Apache License Version 2.0](../LICENSE)