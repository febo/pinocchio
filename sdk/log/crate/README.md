# <img height="70" alt="pinocchio-log" src="https://github.com/user-attachments/assets/caee2220-d11b-4b6a-aefd-6f6bd9815b73"/>

Lightweight log utility for Solana programs.

## Overview

Currently, logging messages that require formatting are a bit heavy on the CU consumption. There are two aspects when comes to determining the cost of a log message:

1. `base cost`: this is the cost of the log syscall. It will either be the [`syscall_base_cost`](https://github.com/anza-xyz/agave/blob/master/compute-budget/src/compute_budget.rs#L167) (currently `100` CU) or a number of CUs equal to the length of the message, whichever value is higher.

2. `formatting cost`: the compute units required to format the message. This is variable and depends on the number and type of the arguments. Formatting is performed using Rust built-in `format!` routines, which in turn use `format_args!`.

It is known that Rust formatting routines are CPU-intensive for constrained environments. This has been noted on both the `solana-program` [`msg!`](https://docs.rs/solana-program/latest/solana_program/macro.msg.html) documentation and more generally on [rust development](https://github.com/rust-lang/rust/issues/99012).

While the cost related to (1) is *fixed*, in the sense that it does not change with the addition of formatting, it is possible to improve the overall cost of logging a formatted message using a lightweight formatting routine &mdash; this is what this crate does.

This crate defines a lightweight `Logger` type to format log messages and a companion `log!` macro. The logger is a fixed size buffer that can be used to format log messages before sending them to the log output. Any type that implements the `Log` trait can be appended to the logger.

Below is a sample of the improvements observed when formatting log messages, measured in terms of compute units (CU):
| Ouput message                      | `log!` | `msg!`       | Improvement (%) |
|------------------------------------|--------|--------------|-----------------|
| `"Hello world!"`                   | 103    | 103          | -               |
| `"lamports={}"` + `u64`            | 374    | 627 (+253)   | 40%             |
| `"{}"` + `[&str; 2]`               | 384    | 1648 (+1264) | 76%             |
| `"{}"` + `[u64; 2]`                | 601    | 1060 (+459)  | 44%             |
| `"lamports={}"` + `i64`            | 389    | 660 (+271)   | 41%             |
| `"{}"` + `[u8; 32]` (pubkey bytes) | 3147   | 8401 (+5254) | 62%             |

> Note: The improvement in CU is accumulative, meaning that if you are logging multiple `u64` values, there will be a 40% improvement per formatted `u64` value.

## Features

* Zero dependencies and `no_std` crate
* Independent of SDK (i.e., works with `pinocchio`, `solana-program` or `anchor`)
* Support for `&str`, unsigned and signed integer types
* `log!` macro to facilitate log message formatting

## Getting Started

From your project folder:
```bash
cargo add pinocchio-log
```

## Usage

The `Logger` can be used directly:
```rust
use pinocchio_log::logger::Logger;

let mut logger = Logger::<100>::default();
logger.append("Hello ");
logger.append("world!");
logger.log();
```

 or via the `log!` macro:
 ```rust
use pinocchio_log::log

let amount = 1_000_000_000;
log!("transfer amount: {}", amount);
```

Since the formatting routine does not perform additional allocations, the `Logger` type has a fixed size specified on its creation. When using the `log!` macro, it is also possible to specify the size of the logger buffer:

```rust
use pinocchio_log::log

let amount = 1_000_000_000;
log!(100, "transfer amount: {}", amount);
```
## Limitations

Currently the `log!` macro does not offer extra formatting options apart from the placeholder "`{}`" for argument values.

## License

The code is licensed under the [Apache License Version 2.0](LICENSE)
