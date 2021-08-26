//! This crate provides Turbocharger's procedural macros.
//!
//! Please refer to the `turbocharger` crate for how to set this up.

#![forbid(unsafe_code)]
#![allow(unused_imports)]

use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use quote::{format_ident, quote, ToTokens};
use syn::{
 parse_macro_input, Data, DeriveInput, Expr, Fields, FieldsNamed, Ident, LitStr, Meta, NestedMeta,
 Token, Type,
};

/// Add this on a `pub async fn` to make it available (over the network) to the JS frontend.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn backend(
 args: proc_macro::TokenStream,
 input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
 // let input = parse_macro_input!(input as DeriveInput);
 // dbg!(input);

 proc_macro::TokenStream::from(quote! {
  fn foo(){}
 })
}
