use std::sync::LazyLock;

use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str,
    punctuated::Punctuated,
    Error, Expr, LitInt, LitStr, Token,
};

/// Regex pattern to match placeholders in the format string.
static PLACEHOLDER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{.*?\}").unwrap());

/// The default buffer size for the logger.
const DEFAULT_BUFFER_SIZE: &str = "200";

/// Represents the input arguments to the `log!` macro.
struct LogArgs {
    /// The length of the buffer to use for the logger.
    ///
    /// This does not have effect when the literal `str` does
    /// not have value placeholders.
    buffer_len: LitInt,

    /// The literal formatting string passed to the macro.
    ///
    /// The `str` might have value placeholders. While this is
    /// not a requirement, the number of placeholders must
    /// match the number of args.
    format_string: LitStr,

    /// The arguments passed to the macro.
    ///
    /// The arguments represent the values to replace the
    /// placeholders on the format `str`. Valid values must implement
    /// the [`Log`] trait.
    args: Punctuated<Expr, Token![,]>,
}

impl Parse for LogArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Optional buffer length.
        let buffer_len = if input.peek(LitInt) {
            let literal = input.parse()?;
            // Parse the comma after the buffer length.
            input.parse::<Token![,]>()?;
            literal
        } else {
            parse_str::<LitInt>(DEFAULT_BUFFER_SIZE)?
        };

        let format_string = input.parse()?;
        // Check if there are any arguments passed to the macro.
        let args = if input.is_empty() {
            Punctuated::new()
        } else {
            input.parse::<Token![,]>()?;
            Punctuated::parse_terminated(input)?
        };

        Ok(LogArgs {
            buffer_len,
            format_string,
            args,
        })
    }
}

/// Companion `log!` macro for `pinocchio-log`.
///
/// The macro automates the creation of a `Logger` object to log a message.
/// It support a limited subset of the [`format!`](https://doc.rust-lang.org/std/fmt/) syntax.
/// The macro parses the format string at compile time and generates the calls to a `Logger`
/// object to generate the corresponding formatted message.
///
/// # Arguments
///
/// - `buffer_len`: The length of the buffer to use for the logger (default to `200`). This is an optional argument.
/// - `format_string`: The literal string to log. This string can contain placeholders `{}` to be replaced by the arguments.
/// - `args`: The arguments to replace the placeholders in the format string. The arguments must implement the `Log` trait.
#[proc_macro]
pub fn log(input: TokenStream) -> TokenStream {
    // Parse the input into a `LogArgs`.
    let LogArgs {
        buffer_len,
        format_string,
        args,
    } = parse_macro_input!(input as LogArgs);
    let parsed_string = format_string.value();

    let placeholders: Vec<_> = PLACEHOLDER
        .find_iter(&parsed_string)
        .map(|m| m.as_str())
        .collect();

    // Check if there is an argument for each `{}` placeholder.
    if placeholders.len() != args.len() {
        let arg_message = if args.is_empty() {
            "but no arguments were given".to_string()
        } else {
            format!(
                "but there is {} {}",
                args.len(),
                if args.len() == 1 {
                    "argument"
                } else {
                    "arguments"
                }
            )
        };

        return Error::new_spanned(
            format_string,
            format!(
                "{} positional arguments in format string, {}",
                placeholders.len(),
                arg_message
            ),
        )
        .to_compile_error()
        .into();
    }

    if !placeholders.is_empty() {
        // The parts of the format string with the placeholders replaced by arguments.
        let mut replaced_parts = Vec::new();

        let parts: Vec<&str> = PLACEHOLDER.split(&parsed_string).collect();
        let part_iter = parts.iter();

        let mut arg_iter = args.iter();
        let mut ph_iter = placeholders.iter();

        // Replace each occurrence of `{}` with their corresponding argument value.
        for part in part_iter {
            if !part.is_empty() {
                replaced_parts.push(quote! { logger.append(#part) });
            }

            if let Some(arg) = arg_iter.next() {
                // The number of placeholders was validated to be the same as
                // the number of arguments, so this should never panic.
                let placeholder = ph_iter.next().unwrap();

                match *placeholder {
                    "{}" => {
                        replaced_parts.push(quote! { logger.append(#arg) });
                    }
                    value if value.starts_with("{:.") => {
                        let precision =
                            if let Ok(precision) = value[3..value.len() - 1].parse::<u8>() {
                                precision
                            } else {
                                return Error::new_spanned(
                                    format_string,
                                    format!("invalid precision format: {}", value),
                                )
                                .to_compile_error()
                                .into();
                            };

                        replaced_parts.push(quote! {
                            logger.append_with_args(
                                #arg,
                                &[pinocchio_log::logger::Argument::Precision(#precision)]
                            )
                        });
                    }
                    _ => {
                        return Error::new_spanned(
                            format_string,
                            format!("invalid placeholder: {}", placeholder),
                        )
                        .to_compile_error()
                        .into();
                    }
                }
            }
        }

        // Generate the output string as a compile-time constant
        TokenStream::from(quote! {
            {
                let mut logger = pinocchio_log::logger::Logger::<#buffer_len>::default();
                #(#replaced_parts;)*
                logger.log();
            }
        })
    } else {
        TokenStream::from(quote! {pinocchio_log::logger::log_message(#format_string.as_bytes());})
    }
}
