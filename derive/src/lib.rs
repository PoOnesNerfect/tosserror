#![allow(
    clippy::blocks_in_if_conditions,
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::manual_find,
    clippy::manual_let_else,
    clippy::manual_map,
    clippy::map_unwrap_or,
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::option_if_let_else,
    clippy::range_plus_one,
    clippy::single_match_else,
    clippy::struct_field_names,
    clippy::too_many_lines,
    clippy::wrong_self_convention
)]

extern crate proc_macro;

mod ast;
mod attr;
mod expand;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Generates helper traits for enum variants.
///
/// ### Attributes:
///
/// `#[visibility]`
///
/// sets visibility of the generated traits.
///
/// Example:
///
/// ```ignore
/// use thiserror::Error;
/// use tosserror::Toss;
///
/// #[derive(Error, Toss, Debug)]
/// #[visibility(pub(crate))] // sets visibility of the generated traits to pub(crate)
/// pub enum Error {
///     ...
/// }
/// ```
///
/// <br>
///
/// `#[prefix]`
///
/// sets custom prefix for the generated traits.
///
/// Example:
///
/// ```ignore
/// use thiserror::Error;
/// use tosserror::Toss;
///
/// #[derive(Error, Toss, Debug)]
/// #[prefix(invalid)] // sets custom prefix `invalid` for the generated traits
/// pub enum Error {
///     Io { ... } // `.toss_io()` becomes `.toss_invalid_io()`
/// }
/// ```
///
/// <br>
///
/// `#[backtrace]`, `#[source]`, `#[from]`
///
/// these are not custom attributes for tosserror. They are used to detect source fields for `thiserror::Error`.
#[proc_macro_derive(Toss, attributes(backtrace, source, from, visibility, prefix))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
