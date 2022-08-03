// Originally from https://github.com/dtolnay/automod 1.0.4
// Licensed under either of Apache License, Version 2.0 or MIT license at your option.

mod error {
 use std::ffi::OsString;
 use std::fmt::{self, Display};
 use std::io;

 pub enum Error {
  Io(io::Error),
  Utf8(OsString),
  Empty,
 }

 pub type Result<T> = std::result::Result<T, Error>;

 impl Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
   use self::Error::*;

   match self {
    Io(err) => err.fmt(f),
    Utf8(name) => write!(f, "unsupported non-utf8 file name: {}", name.to_string_lossy(),),
    Empty => f.write_str("no source files found"),
   }
  }
 }

 impl From<io::Error> for Error {
  fn from(err: io::Error) -> Self {
   Error::Io(err)
  }
 }
}

use self::error::{Error, Result};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, LitStr, Token, Visibility};

struct Arg {
 vis: Visibility,
 useness: Option<Token![use]>,
 path: LitStr,
}

impl Parse for Arg {
 fn parse(input: ParseStream) -> syn::Result<Self> {
  Ok(Arg { vis: input.parse()?, useness: input.parse()?, path: input.parse()? })
 }
}

pub fn dir(input: TokenStream) -> TokenStream {
 let input = parse_macro_input!(input as Arg);
 let vis = &input.vis;
 let useness = input.useness.is_some();
 let rel_path = input.path.value();

 let dir = match env::var_os("CARGO_MANIFEST_DIR") {
  Some(manifest_dir) => PathBuf::from(manifest_dir).join(rel_path),
  None => PathBuf::from(rel_path),
 };

 let expanded = match source_file_names(dir) {
  Ok(names) => names.into_iter().map(|name| mod_item(vis, useness, name)).collect(),
  Err(err) => syn::Error::new(Span::call_site(), err).to_compile_error(),
 };

 TokenStream::from(expanded)
}

fn mod_item(vis: &Visibility, useness: bool, name: String) -> TokenStream2 {
 if name.contains('-') {
  let path = format!("{}.rs", name);
  let ident = Ident::new(&name.replace('-', "_"), Span::call_site());
  if useness {
   quote! {
       #[path = #path]
       #vis mod #ident;
       #vis use self::#ident::*;
   }
  } else {
   quote! {
       #[path = #path]
       #vis mod #ident;
   }
  }
 } else {
  let ident = Ident::new(&name, Span::call_site());
  if useness {
   quote! {
       #vis mod #ident;
       #vis use self::#ident::*;
   }
  } else {
   quote! {
       #vis mod #ident;
   }
  }
 }
}

fn source_file_names<P: AsRef<Path>>(dir: P) -> Result<Vec<String>> {
 let mut names = Vec::new();
 let mut failures = Vec::new();

 for entry in fs::read_dir(dir)? {
  let entry = entry?;
  if !entry.file_type()?.is_file() {
   continue;
  }

  let file_name = entry.file_name();
  if file_name == "mod.rs" || file_name == "lib.rs" || file_name == "main.rs" {
   continue;
  }

  let path = Path::new(&file_name);
  if path.extension() == Some(OsStr::new("rs")) {
   match file_name.into_string() {
    Ok(mut utf8) => {
     utf8.truncate(utf8.len() - ".rs".len());
     names.push(utf8);
    }
    Err(non_utf8) => {
     failures.push(non_utf8);
    }
   }
  }
 }

 failures.sort();
 if let Some(failure) = failures.into_iter().next() {
  return Err(Error::Utf8(failure));
 }

 if names.is_empty() {
  return Err(Error::Empty);
 }

 names.sort();
 Ok(names)
}
