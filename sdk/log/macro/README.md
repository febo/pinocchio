# `pinocchio-log-macro`

Companion `log!` macro for `pinocchio-log`. It automates the creation of a `Logger` object to log a message. It support a limited subset of the [`format!`](https://doc.rust-lang.org/std/fmt/) syntax. The macro parses the format string at compile time and generates the calls to a `Logger` object to generate the corresponding formatted message.

## Usage

The macro works very similar to `solana-program` [`msg!`](https://docs.rs/solana-program/latest/solana_program/macro.msg.html) macro.

To output a simple message (static `str`):
```rust
use pinocchio_log::log

log!("a simple log");
```

To ouput a formatted message:
```rust
use pinocchio_log::log

let amount = 1_000_000_000;
log!("transfer amount: {}", amount);
```

Since a `Logger` size is statically determined, messages are limited to `200` length by default. When logging larger messages, it is possible to increase the logger buffer:
```rust
use pinocchio_log::log

let very_long_message = "...";
log!(500, "message: {}", very_long_message);
```

## License

The code is licensed under the [Apache License Version 2.0](LICENSE)