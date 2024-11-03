use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str,
    punctuated::Punctuated,
    Error, Expr, LitInt, LitStr, Token,
};

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

#[proc_macro]
pub fn log(input: TokenStream) -> TokenStream {
    // Parse the input into a `LogArgs`.
    let LogArgs {
        buffer_len,
        format_string,
        args,
    } = parse_macro_input!(input as LogArgs);
    let parsed_string = format_string.value();

    // Check if there are any `{}` placeholders in the format string.
    //
    // When the format string has placeholders, the list of arguments must
    // not be empty. The number of placehilders will be validated later.
    let needs_formatting = parsed_string.contains("{}");

    if !(needs_formatting || args.is_empty()) {
        return Error::new_spanned(
            format_string,
            "the format string must contain a `{}` placeholder for each value.",
        )
        .to_compile_error()
        .into();
    }

    if needs_formatting {
        // The parts of the format string with the placeholders replaced by arguments.
        let mut replaced_parts = Vec::new();
        // The number of placeholders in the format string.
        let mut part_count = 0;
        // The number of arguments passed to the macro.
        let mut arg_count = 0;

        let part_iter = parsed_string.split("{}").peekable();
        let mut arg_iter = args.iter();

        // Replace each occurrence of `{}` with their corresponding argument value.
        for part in part_iter {
            replaced_parts.push(quote! { logger.append(#part) });
            part_count += 1;

            if let Some(arg) = arg_iter.next() {
                replaced_parts.push(quote! { logger.append(#arg) });
                arg_count += 1;
            }
        }

        if (part_count - 1) != arg_count {
            let arg_message = if arg_count == 0 {
                "but no arguments were given".to_string()
            } else {
                format!(
                    "but there is {} {}",
                    arg_count,
                    if arg_count == 1 {
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
                    part_count - 1,
                    arg_message
                ),
            )
            .to_compile_error()
            .into();
        }

        // Generate the output string as a compile-time constant
        TokenStream::from(quote! {
            {
                let mut logger = ::pinocchio_logger::Logger::<#buffer_len>::default();
                #(#replaced_parts;)*
                logger.log();
            }
        })
    } else {
        TokenStream::from(quote! {log(#format_string.as_bytes());})
    }
}
