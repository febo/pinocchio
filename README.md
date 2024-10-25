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
* access to system accounts (`sysvars`)
* cross-program invocation

## Features

* Zero dependencies and `no_std` crate
* Efficient `entrypoint!` macro – no copies or allocations
* Improved CU consumption of cross-program invocations

## Getting started

From your project folder:

```bash
cargo add pinocchio
```

Pinocchio provides two different entrypoint macros: an `entrypoint` that looks similar to the "standard" one found in `solana-program` and a lightweight `lazy_entrypoint`. The main difference between them is how much work the entrypoint performs. While the `entrypoint` parsers the whole input and provide the `program_id`, `accounts` and `instruction_data` separately, the `lazy_entrypoint` only wraps the input at first. It then provides methods to parse the input on demand. The benefit in this case is that you have more control when the parsing is happening &mdash; even whether the parsing is needed or not.

The `lazy_entrypoint` is suitable for programs that have a single or very few instructions, since it requires the program to handle the parsing, which can become complex as the number of instructions increases. For "larger" programs, the `entrypoint` will likely be easier and more efficient to use.

> ⚠️ **Note:**
> In both cases you should use the types from the `pinocchio` crate instead of `solana-program`. If you need to invoke a different program, you will need to redefine its instruction builder to create an equivalent instruction data using `pinocchio` types.

### 🚪 `entrypoint!`

To use the `entrypoint!` macro, use the following in your entrypoint definition:
```rust
use pinocchio::{
  account_info::AccountInfo,
  entrypoint,
  msg,
  ProgramResult
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

The information from the input is parsed into their own entities:

* `program_id`: the `ID` of the program being called
* `accounts`: the accounts received
* `instruction_data`: data for the instruction

### 🚪 `lazy_entrypoint!`

To use the `lazy_entrypoint!` macro, use the following in your entrypoint definition:
```rust
use pinocchio::{
  lazy_entrypoint,
  lazy_entrypoint::InstructionContext,
  msg,
  ProgramResult
};

lazy_entrypoint!(process_instruction);

pub fn process_instruction(
  mut context: InstructionContext,
) -> ProgramResult {
  msg!("Hello from my lazy program!");
  Ok(())
}
```

The `InstructionContext` provides on-demand access to the information of the input:

* `available()`: number of available accounts
* `next_account()`: parsers the next available account (can be used as many times as accounts available)
* `instruction_data()`: parsers the intruction data and program id

## License

The code is licensed under the [Apache License Version 2.0](LICENSE)

The library in this repository is based/includes code from:
* [`nitrate`](https://github.com/nifty-oss/nitrate)
* [`solana-nostd-entrypoint`](https://github.com/cavemanloverboy/solana-nostd-entrypoint/tree/main)
* [`solana-program`](https://github.com/anza-xyz/agave/tree/master/sdk/program)
