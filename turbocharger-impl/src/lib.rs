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

#[proc_macro_attribute]
#[proc_macro_error]
pub fn server_only(
 _args: proc_macro::TokenStream,
 input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
 let orig_fn = parse_macro_input!(input as syn::ItemFn);
 proc_macro::TokenStream::from(quote! {
  #[cfg(not(target_arch = "wasm32"))]
  #[allow(dead_code)]
  #orig_fn
 })
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn wasm_only(
 _args: proc_macro::TokenStream,
 input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
 let orig_fn = parse_macro_input!(input as syn::ItemFn);
 proc_macro::TokenStream::from(quote! {
  #[cfg(target_arch = "wasm32")]
  #[wasm_bindgen(js_class = wasm_only)]
  impl wasm_only {
   #[wasm_bindgen]
   #orig_fn
  }
 })
}

/// Add this on a `pub async fn` to make it available (over the network) to the JS frontend.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn backend(
 _args: proc_macro::TokenStream,
 input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
 let orig_fn = parse_macro_input!(input as syn::ItemFn);

 let orig_fn_ident = orig_fn.sig.ident.clone();
 let orig_fn_string = orig_fn_ident.to_string();
 let orig_fn_params = orig_fn.sig.inputs.clone();
 let orig_fn_ret_ty = orig_fn.sig.output.clone();

 let tuple_indexes = (0..orig_fn_params.len()).map(|i| syn::Index::from(i));
 let orig_fn_param_names = orig_fn_params.iter().map(|p| match p {
  syn::FnArg::Receiver(_) => abort_call_site!("I don't know what to do with `self` here."),
  syn::FnArg::Typed(pattype) => match *pattype.pat.clone() {
   syn::Pat::Ident(i) => i.ident,
   _ => abort_call_site!("Eeps"),
  },
 });
 let orig_fn_param_tys = orig_fn_params.iter().map(|p| match p {
  syn::FnArg::Receiver(_) => abort_call_site!("I don't know what to do with `self` here."),
  syn::FnArg::Typed(pattype) => &pattype.ty,
 });
 let orig_fn_param_tys_cloned = orig_fn_param_tys.clone();

 let orig_fn_params_maybe_comma = if orig_fn_params.len() == 0 {
  quote! {}
 } else {
  quote! { , }
 };

 let mod_name =
  Ident::new(&format!("_TURBOCHARGER_{}", orig_fn_string), proc_macro2::Span::call_site());
 let dispatch =
  Ident::new(&format!("_TURBOCHARGER_DISPATCH_{}", orig_fn_string), proc_macro2::Span::call_site());
 let req =
  Ident::new(&format!("_TURBOCHARGER_REQ_{}", orig_fn_string), proc_macro2::Span::call_site());
 let resp =
  Ident::new(&format!("_TURBOCHARGER_RESP_{}", orig_fn_string), proc_macro2::Span::call_site());

 proc_macro::TokenStream::from(quote! {
  #[cfg(not(target_arch = "wasm32"))]
  #orig_fn

  #[cfg(not(target_arch = "wasm32"))]
  mod #mod_name {
   use ::turbocharger::typetag;
   #[::turbocharger::typetag::serde(name = #orig_fn_string)]
   #[::turbocharger::async_trait]
   impl ::turbocharger::RPC for super::#dispatch {
    async fn execute(&self) -> Vec<u8> {
     let response = super::#resp {
      txid: self.txid,
      result: super::#orig_fn_ident(#( self.params. #tuple_indexes .clone() ),*).await,
     };
     ::turbocharger::bincode::serialize(&response).unwrap()
    }
   }
  }

  #[cfg(target_arch = "wasm32")]
  #[wasm_bindgen(js_class = backend)]
  impl backend {
   #[wasm_bindgen]
   pub async fn #orig_fn_ident(#orig_fn_params) -> String {
    {
     let t = ::turbocharger::_Transaction::new();
     let txid = t.txid;
     let req = ::turbocharger::bincode::serialize(&#req {
      typetag_const_one: 1,
      dispatch_name: #orig_fn_string,
      txid: t.txid,
      params: (#(#orig_fn_param_names),* #orig_fn_params_maybe_comma),
     })
     .unwrap();
     let response = t.run(req).await;
     let #resp { result, .. } =
      ::turbocharger::bincode::deserialize(&response).unwrap();
     console_log!("tx {}: {:?}", txid, result);
     result
    }
   }
  }

  #[allow(non_camel_case_types)]
  #[derive(::turbocharger::serde::Serialize, ::turbocharger::serde::Deserialize)]
  #[serde(crate = "::turbocharger::serde")]
  struct #req {
   typetag_const_one: i64,
   dispatch_name: &'static str,
   txid: i64,
   params: (#(#orig_fn_param_tys),* #orig_fn_params_maybe_comma),
  }

  #[allow(non_camel_case_types)]
  #[derive(::turbocharger::serde::Serialize, ::turbocharger::serde::Deserialize, Debug)]
  #[serde(crate = "::turbocharger::serde")]
  struct #dispatch {
   txid: i64,
   params: (#(#orig_fn_param_tys_cloned),* #orig_fn_params_maybe_comma),
  }

  #[allow(non_camel_case_types)]
  #[derive(::turbocharger::serde::Serialize, ::turbocharger::serde::Deserialize)]
  #[serde(crate = "::turbocharger::serde")]
  struct #resp {
   txid: i64,
   result: String,
  }


 })
}
