<p align="center">
  <a href="https://github.com/anza-xyz/pinocchio">
    <img alt="pinocchio" src="https://github.com/user-attachments/assets/4048fe96-9096-4441-85c3-5deffeb089a6" height="100" />
  </a>
</p>
<h3 align="center">
  <code>pinocchio-adapter</code>
</h3>
<p align="center">
  <a href="https://crates.io/crates/pinocchio-adapter"><img src="https://img.shields.io/crates/v/pinocchio-adapter?logo=rust" /></a>
  <a href="https://docs.rs/pinocchio-adapter/latest/pinocchio_adapter/"><img src="https://img.shields.io/docsrs/pinocchio-adapter?logo=docsdotrs" /></a>
</p>

This crate contains helpers to create [`pinocchio`](https://crates.io/crates/pinocchio) types from Solana SDK types.

This is a `no_std` crate.

> **Note:** The API defined in this crate is subject to change.

## Getting Started

From your project folder:

```bash
cargo add pinocchio-adapter
```

This will add the `pinocchio-adapter` dependency to your `Cargo.toml` file.

## Examples

Creating an `pinocchio::account_info::AccountInfo` from `solana_account_info::AccountInfo`:
```rust
let [account_info] = accounts else {
    return Err(ProgramError::NotEnoughAccountKeys);
};

// SAFETY: `account_info` is a valid reference to an `AccountInfo` and it is
// not used directly after the adapter is created.
let account_adapter = unsafe { AccountInfoAdapter::new(account_info) };

// An adapter can be dereferenced to a `pinocchio::account_info::AccountInfo`
use_pinocchio_account_info(&from_adapter);
```

## License

The code is licensed under the [Apache License Version 2.0](../LICENSE)
