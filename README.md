<h1 align="center">
  <code>pinocchio</code>
</h1>
<p align="center">
  <img width="400" alt="Limestone" src="https://github.com/user-attachments/assets/3a1894b4-403f-4c35-90aa-548e7672fe90" />
</p>
<p align="center">
  Create Solana programs with no dependencies attached.
</p>

<p align="center">
  <a href="https://github.com/febo/pinocchio/actions/workflows/main.yml"><img src="https://img.shields.io/github/actions/workflow/status/febo/pinocchio/main.yml?logo=GitHub" /></a>
  <a href="https://crates.io/crates/pinocchio"><img src="https://img.shields.io/crates/v/pinocchio?logo=rust" /></a>
</p>

<p align="right">
<i>I've got no dependencies</i><br />
<i>To hold me down</i><br />
<i>To make me fret</i><br />
<i>Or make me frown</i><br />
<i>I had dependencies</i><br />
<i>But now I'm free</i><br />
<i>There are no dependencies on me</i>
</p>

## Overview

Pinocchio is a zero-dependency library to create Solana programs in Rust. It takes advantage of the way SBF loaders serialize the program input parameters into a byte array that is then passed to the program's entrypoint to define zero-copy types to read the input. Since the communication between a program and SBF loader &mdash; either at the first time the program is called or when one program invokes the instructions of another program &mdash; is done via a byte array, a program can define its own types. This completely eliminates the dependency on the `solana-program` crate, which in turn mitigates dependency issues by having a crate specifically designed to create on-chain programs.

Pinocchio can be used as a replacement for [`solana-program`](https://crates.io/crates/solana-program) to write on-chain programs.

The library defines:
* program entrypoint
* core data types
* logging macros
* `syscall` functions
* access to system accounts (`sysvar`)
* cross-program invocation

## Features

* `no_std` crate.
* Efficient `entrypoint!` definition – no copies or no allocations.
* Improved CU consumption of cross-program invocations.

## Getting started

From your project folder:

```bash
cargo add pinocchio
```

On your entrypoint definition:
```rust
use pinocchio::{
  account_info::AccountInfo,
  entrypoint,
  entrypoint::ProgramResult,
  msg,
  pubkey::Pubkey
};

entrypoint!(process_instruction);

pub fn process_instruction(
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  instruction_data: &[u8],
) -> ProgramResult {
  msg!("Hello from my program!");
  Ok(())
}
```

> [!IMPORTANT]
> You should use the types from the `pinocchio` crate instead of `solana-program`. If you need to invoke a different program, you will need to redefine its instruction builder to create an equivalent instruction data using `pinocchio` types.

## License

The code is licensed under the [Apache License Version 2.0](LICENSE)

The library in this repository is based/includes code from:
* [`nitrate`](https://github.com/nifty-oss/nitrate)
* [`solana-nostd-entrypoint`](https://github.com/cavemanloverboy/solana-nostd-entrypoint/tree/main)
* [`solana-program`](https://github.com/anza-xyz/agave/tree/master/sdk/program)
