<h1 align="center">
  <code>pinocchio</code>
</h1>
<p align="center">
  <img width="400" alt="Limestone" src="https://github.com/user-attachments/assets/3a1894b4-403f-4c35-90aa-548e7672fe90" />
</p>
<p align="center">
  Create Solana programs with minimal dependencies attached.
</p>

<p align="center">
  <a href="https://github.com/anza-xyz/pinocchio/actions/workflows/main.yml"><img src="https://img.shields.io/github/actions/workflow/status/anza-xyz/pinocchio/main.yml?logo=GitHub" /></a>
  <a href="https://crates.io/crates/pinocchio"><img src="https://img.shields.io/crates/v/pinocchio?logo=rust" /></a>
</p>

## Overview

Pinocchio is a library to create Solana programs in Rust. It takes advantage of the way SBF loaders serialize the program input parameters into a byte array that is then passed to the program's entrypoint to define zero-copy types to read the input. Since the communication between a program and SBF loader &mdash; either at the first time the program is called or when one program invokes the instructions of another program &mdash; is done via a byte array, a program can define its own types. This completely eliminates the dependency on the `solana-program` crate, which in turn mitigates dependency issues by having a crate specifically designed to create on-chain programs.

As a result, Pinocchio can be used as a replacement for [`solana-program`](https://crates.io/crates/solana-program) to write on-chain programs, which are optimized in terms of both compute units consumption and binary size.

The library defines:
* program entrypoint
* core data types
* logging macros
* `syscall` functions
* access to system accounts (`sysvars`)
* cross-program invocation

## Features

* Minimal dependencies and `no_std` crate
* Efficient `entrypoint!` macro – no copies or allocations
* Improved CU consumption of cross-program invocations

## Getting started

From your project folder:

```bash
cargo add pinocchio
```

This will add `pinocchio` as a dependency to your project.

## Defining the program entrypoint

A Solana program needs to define an entrypoint, which will be called by the runtime to begin the program execution. The `entrypoint!` macro emits the common boilerplate to set up the program entrypoint. The macro will also set up [global allocator](https://doc.rust-lang.org/stable/core/alloc/trait.GlobalAlloc.html) and [panic handler](https://doc.rust-lang.org/nomicon/panic-handler.html) using the [default_allocator!](https://docs.rs/pinocchio/latest/pinocchio/macro.default_allocator.html) and [default_panic_handler!](https://docs.rs/pinocchio/latest/pinocchio/macro.default_panic_handler.html) macros.

The [`entrypoint!`](https://docs.rs/pinocchio/latest/pinocchio/macro.entrypoint.html) is a convenience macro that invokes three other macros to set all symbols required for a program execution:

* [`program_entrypoint!`](https://docs.rs/pinocchio/latest/pinocchio/macro.program_entrypoint.html): declares the program entrypoint
* [`default_allocator!`](https://docs.rs/pinocchio/latest/pinocchio/macro.default_allocator.html): declares the default (bump) global allocator
* [`default_panic_hanlder!`](https://docs.rs/pinocchio/latest/pinocchio/macro.default_panic_handler.html): declares the default panic handler

To use the `entrypoint!` macro, use the following in your entrypoint definition:
```rust
use pinocchio::{
  account_info::AccountInfo,
  entrypoint,
  msg,
  ProgramResult,
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

`pinocchio` also offers variations of the program entrypoint (`lazy_program_allocator`) and global allocator (`no_allocator`). In order to use these, the program needs to specify the program entrypoint, global allocator and panic handler individually. The `entrypoint!` macro is equivalent to writing:
```rust
program_entrypoint!(process_instruction);
default_allocator!();
default_panic_handler!();
```
Any of these macros can be replaced by other implementations and `pinocchio` offers a couple of variants for this.

📌 [`lazy_program_entrypoint!`](https://docs.rs/pinocchio/latest/pinocchio/macro.lazy_program_entrypoint.html)

The `entrypoint!` macro looks similar to the "standard" one found in `solana-program`. It parsers the whole input and provides the `program_id`, `accounts` and `instruction_data` separately. This consumes compute units before the program begins its execution. In some cases, it is beneficial for a program to have more control when the input parsing is happening, even whether the parsing is needed or not &mdash; this is the purpose of the [`lazy_program_entrypoint!`](https://docs.rs/pinocchio/latest/pinocchio/macro.lazy_program_entrypoint.html) macro. This macro only wraps the program input and provides methods to parse the input on demand.

The `lazy_entrypoint` is suitable for programs that have a single or very few instructions, since it requires the program to handle the parsing, which can become complex as the number of instructions increases. For *larger* programs, the [`program_entrypoint!`](https://docs.rs/pinocchio/latest/pinocchio/macro.program_entrypoint.html) will likely be easier and more efficient to use.

To use the `lazy_program_entrypoint!` macro, use the following in your entrypoint definition:
```rust
use pinocchio::{
  default_allocator,
  default_panic_handler,
  entrypoint::InstructionContext,
  lazy_program_entrypoint,
  msg,
  ProgramResult
};

lazy_program_entrypoint!(process_instruction);
default_allocator!();
default_panic_handler!();

pub fn process_instruction(
  mut context: InstructionContext
) -> ProgramResult {
    msg!("Hello from my lazy program!");
    Ok(())
}
```

The `InstructionContext` provides on-demand access to the information of the input:

* `available()`: number of available accounts
* `next_account()`: parsers the next available account (can be used as many times as accounts available)
* `instruction_data()`: parsers the intruction data
* `program_id()`: parsers the program id

> ⚠️ **Note:**
> The `lazy_program_entrypoint!` does not set up a global allocator nor a panic handler. A program should explicitly use one of the provided macros to set them up or include its own implementation.

📌 [`no_allocator!`](https://docs.rs/pinocchio/latest/pinocchio/macro.no_allocator.html)

When writing programs, it can be useful to make sure the program does not attempt to make any allocations. For this cases, `pinocchio` includes a [`no_allocator!`](https://docs.rs/pinocchio/latest/pinocchio/macro.no_allocator.html) macro that set a global allocator just panics at any attempt to allocate memory.

To use the `no_allocator!` macro, use the following in your entrypoint definition:
```rust
use pinocchio::{
  account_info::AccountInfo,
  default_panic_handler,
  msg,
  no_allocator,
  program_entrypoint,
  ProgramResult,
  pubkey::Pubkey
};

program_entrypoint!(process_instruction);
default_panic_handler!();
no_allocator!();

pub fn process_instruction(
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  instruction_data: &[u8],
) -> ProgramResult {
  msg!("Hello from `no_std` program!");
  Ok(())
}
```
> ⚠️ **Note:**
> The `no_allocator!` macro can also be used in combination with the `lazy_program_entrypoint!`.

## Crate feature: `std`

By default, `pinocchio` is a `no_std` crate. This means that it does not use any code from the standard (`std`) library. While this does not affect how `pinocchio` is used, there is a one particular apparent difference. In a `no_std` environment, the `msg!` macro does not provide any formatting options since the `format!` macro requires the `std` library. In order to use `msg!` with formatting, the `std` feature should be enable when adding `pinocchio` as a dependency:
```
pinocchio = { version = "0.7.0", features = ["std"] }
```

Instead of enabling the `std` feature to be able to format log messages with `msg!`, it is recommented to use the [`pinocchio-log`](https://crates.io/crates/pinocchio-log) crate. This crate provides a lightweight `log!` macro with better compute units consumption than the standard `format!` macro without requiring the `std` library.

## Advance entrypoint configuration

The symbols emitted by the entrypoint macros &mdash; program entrypoint, global allocator and default panic handler &mdash; can only be defined once globally. If the program crate is also intended to be use as a library, it is common practice to define a Cargo [feature](https://doc.rust-lang.org/cargo/reference/features.html) in your program crate to conditionally enable the module that includes the `entrypoint!` macro invocation. The convention is to name the feature `bpf-entrypoint`.

```rust
#[cfg(feature = "bpf-entrypoint")]
mod entrypoint {
  use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    msg,
    ProgramResult,
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
}
```
When building the program binary, you must enable the `bpf-entrypoint` feature:
```bash
cargo build-sbf --features bpf-entrypoint
```

## License

The code is licensed under the [Apache License Version 2.0](LICENSE)

The library in this repository is based/includes code from:
* [`nitrate`](https://github.com/nifty-oss/nitrate)
* [`solana-nostd-entrypoint`](https://github.com/cavemanloverboy/solana-nostd-entrypoint/tree/main)
* [`solana-program`](https://github.com/anza-xyz/agave/tree/master/sdk/program)
