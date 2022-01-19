//! This crate provides Turbocharger's procedural macros.
//!
//! Please refer to the `turbocharger` crate for how to set this up.

#![forbid(unsafe_code)]

mod extract_result;
mod extract_stream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote};

/// Apply this to an item to make it available on the server target only.
///
/// Only adds `#[cfg(not(target_arch = "wasm32"))]`
#[proc_macro_attribute]
pub fn server_only(
 _args: proc_macro::TokenStream,
 input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
 let orig_item = syn::parse_macro_input!(input as syn::Item);
 proc_macro::TokenStream::from(quote! {
  #[cfg(not(target_arch = "wasm32"))]
  #orig_item
 })
}

/// Apply this to an item to make it available on the wasm target only.
///
/// Only adds `#[cfg(target_arch = "wasm32")]` and ensures `wasm_bindgen::prelude::*` is available.
#[proc_macro_attribute]
pub fn wasm_only(
 _args: proc_macro::TokenStream,
 input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
 let orig_item = syn::parse_macro_input!(input as syn::Item);
 proc_macro::TokenStream::from(quote! {
  #[cfg(target_arch = "wasm32")]
  #[allow(unused_imports)]
  use wasm_bindgen::prelude::*;

  #[cfg(target_arch = "wasm32")]
  #orig_item
 })
}

/// Apply this to a `pub async fn` to make it available (over the network) to the JS frontend.
///
/// Also apply to any `struct`s used in backend function signatures.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn backend(
 _args: proc_macro::TokenStream,
 input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
 backend_item(syn::parse_macro_input!(input as syn::Item)).into()
}

fn backend_item(orig_item: syn::Item) -> proc_macro2::TokenStream {
 match orig_item {
  syn::Item::Fn(orig) => backend_fn(orig),
  syn::Item::Struct(orig) => backend_struct(orig),
  // syn::Item::Mod(orig) => backend_mod(orig),
  _ => abort!(orig_item, "Apply #[backend] to `fn` or `struct`."),
 }
}

// fn backend_mod_item(orig_item: syn::Item) -> proc_macro2::TokenStream {
//  match orig_item {
//   syn::Item::Fn(orig) => backend_fn(orig),
//   syn::Item::Struct(orig) => backend_struct(orig),
//   orig => quote! { #orig },
//  }
// }

// fn backend_mod(orig_mod: syn::ItemMod) -> proc_macro2::TokenStream {
//  let content = orig_mod.content.clone();

//  let items: Vec<_> = content
//   .unwrap_or_else(|| abort!(orig_mod, "Apply #[backend] to a `mod` with a body."))
//   .1
//   .into_iter()
//   .map(backend_mod_item)
//   .collect();

//  quote! { #(#items)* }
// }

fn backend_struct(orig_struct: syn::ItemStruct) -> proc_macro2::TokenStream {
 let syn::ItemStruct { attrs, ident, fields, .. } = orig_struct;

 quote! {
  #[cfg(target_arch = "wasm32")]
  #[allow(unused_imports)]
  use wasm_bindgen::prelude::*;

  #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone, inspectable))]
  #[derive(::turbocharger::serde::Serialize, ::turbocharger::serde::Deserialize, Clone)]
  #(#attrs)*
  #[serde(crate = "::turbocharger::serde")]
  pub struct #ident #fields

  #[cfg(target_arch = "wasm32")]
  #[wasm_bindgen]
  impl #ident {
   #[wasm_bindgen(constructor)]
   pub fn new() -> #ident {
    #ident::default()
   }
  }
 }
}

fn backend_fn(orig_fn: syn::ItemFn) -> proc_macro2::TokenStream {
 let orig_fn_ident = orig_fn.sig.ident.clone();
 let orig_fn_string = orig_fn_ident.to_string();
 let orig_fn_params = orig_fn.sig.inputs.clone();

 let mod_name = format_ident!("_TURBOCHARGER_{}", orig_fn_ident);
 let store_name = format_ident!("_TURBOCHARGER_STORE_{}", orig_fn_ident);
 let dispatch = format_ident!("_TURBOCHARGER_DISPATCH_{}", orig_fn_ident);
 let req = format_ident!("_TURBOCHARGER_REQ_{}", orig_fn_ident);
 let resp = format_ident!("_TURBOCHARGER_RESP_{}", orig_fn_ident);
 let impl_fn_ident = format_ident!("_TURBOCHARGER_IMPL_{}", orig_fn_ident);
 let remote_fn_ident = format_ident!("remote_{}", orig_fn_ident);

 let orig_fn_ret_ty = match orig_fn.sig.output.clone() {
  syn::ReturnType::Default => None,
  syn::ReturnType::Type(_, path) => Some(*path),
 };
 let result_inner_ty = orig_fn_ret_ty.clone().map(extract_result::inner_ty).flatten();
 let stream_inner_ty = orig_fn_ret_ty.clone().map(extract_stream::inner_ty).flatten();

 let orig_fn_ret_ty = match (&orig_fn_ret_ty, &stream_inner_ty) {
  (Some(ty), None) => quote! { #ty },
  (Some(_), Some(ty)) => quote! { impl ::turbocharger::futures::stream::Stream<Item = #ty > },
  (None, None) => quote! { () },
  _ => abort!(orig_fn_ret_ty, "Really confused about this return type!"),
 };

 let mut orig_fn = orig_fn;

 orig_fn.sig.output = dbg!(syn::parse2(quote! { -> #orig_fn_ret_ty })).unwrap();

 let bindgen_ret_ty = match (&result_inner_ty, &stream_inner_ty) {
  (Some(ty), None) => quote! { Result<#ty, JsValue> },
  (None, Some(_ty)) => quote! { #store_name },
  (None, None) => quote! { #orig_fn_ret_ty },
  _ => abort!(orig_fn_ret_ty, "Only one of `Result` or `Stream` is allowed."),
 };
 let serialize_ret_ty = match (&result_inner_ty, &stream_inner_ty) {
  (Some(ty), None) => quote! { Result<#ty, String> },
  (None, Some(ty)) => quote! { #ty },
  (None, None) => quote! { #orig_fn_ret_ty },
  _ => abort!(orig_fn_ret_ty, "Only one of `Result` or `Stream` is allowed."),
 };
 let maybe_map_err_string = match &result_inner_ty {
  Some(_) => quote! { .map_err(|e| e.to_string()) },
  None => quote! {},
 };
 let maybe_map_err_jsvalue = match &result_inner_ty {
  Some(_) => quote! { .map_err(|e| ::turbocharger::js_sys::Error::new(&e).into()) },
  None => quote! {},
 };

 let tuple_indexes = (0..orig_fn_params.len()).map(syn::Index::from);
 let orig_fn_param_names: Vec<_> = orig_fn_params
  .iter()
  .map(|p| match p {
   syn::FnArg::Receiver(_) => abort!(p, "I don't know what to do with `self` here."),
   syn::FnArg::Typed(pattype) => match *pattype.pat.clone() {
    syn::Pat::Ident(i) => i.ident,
    _ => abort!(pattype, "Parameter name is not Ident"),
   },
  })
  .collect();

 let orig_fn_param_tys: Vec<_> = orig_fn_params
  .iter()
  .map(|p| match p {
   syn::FnArg::Receiver(_) => abort!(p, "I don't know what to do with `self` here."),
   syn::FnArg::Typed(pattype) => &pattype.ty,
  })
  .collect();

 let orig_fn_params_maybe_comma = if orig_fn_params.is_empty() {
  quote! {}
 } else {
  quote! { , }
 };

 let executebody = match &stream_inner_ty {
  Some(_ty) => quote! {
   use ::turbocharger::futures::stream::StreamExt;
   let stream = super::#orig_fn_ident(#( self.params. #tuple_indexes .clone() ),*);
   ::turbocharger::futures::pin_mut!(stream);
   while let Some(result) = stream.next().await {
    let response = super::#resp {
     txid: self.txid,
     result
    };
    sender(::turbocharger::bincode::serialize(&response).unwrap());
   }
  },
  None => quote! {
   let result = super::#orig_fn_ident(#( self.params. #tuple_indexes .clone() ),*).await #maybe_map_err_string;
   let response = super::#resp {
    txid: self.txid,
    result
   };
   sender(::turbocharger::bincode::serialize(&response).unwrap());
  },
 };

 let wasm_side = match &stream_inner_ty {
  Some(ty) => quote! {
   #[cfg(target_arch = "wasm32")]
   #[allow(non_camel_case_types)]
   #[derive(Default)]
   #[wasm_bindgen]
   pub struct #store_name {
    value: Option< #ty >,
    subscriptions: Vec<::turbocharger::js_sys::Function>,
   }

   #[cfg(target_arch = "wasm32")]
   #[wasm_bindgen]
   impl #store_name {
    #[wasm_bindgen]
    pub fn subscribe(&mut self, subscription: ::turbocharger::js_sys::Function) -> JsValue {
     if let Some(value) = &self.value {
      let this = JsValue::null();
      subscription.call1(&this, &value.clone().into()).ok();
     }
     self.subscriptions.push(subscription);

     Closure::wrap(Box::new(move || {
      dbg!("unsubscribe called!!");
     }) as Box<dyn Fn()>)
     .into_js_value()
    }
   }

   #[cfg(target_arch = "wasm32")]
   #[wasm_bindgen]
   pub fn #orig_fn_ident(#orig_fn_params) -> #bindgen_ret_ty {
    let tx = ::turbocharger::_Transaction::new();
    let req = ::turbocharger::bincode::serialize(&#req {
     typetag_const_one: 1,
     dispatch_name: #orig_fn_string,
     txid: tx.txid,
     params: (#( #orig_fn_param_names ),* #orig_fn_params_maybe_comma),
    })
    .unwrap();
    tx.send_ws(req);
    tx.set_sender(Box::new(move |response| {
     let #resp { result, .. } =
      ::turbocharger::bincode::deserialize(&response).unwrap();
     ::turbocharger::console_log!("got result: {:?}", result);
    }));
    #store_name ::default()
   }
  },
  None => quote! {
   #[cfg(target_arch = "wasm32")]
   #[wasm_bindgen]
   pub async fn #orig_fn_ident(#orig_fn_params) -> #bindgen_ret_ty {
    #impl_fn_ident(#( #orig_fn_param_names ),*) .await #maybe_map_err_jsvalue
   }

   #[cfg(target_arch = "wasm32")]
   #[allow(non_snake_case)]
   async fn #impl_fn_ident(#orig_fn_params) -> #serialize_ret_ty {
    let tx = ::turbocharger::_Transaction::new();
    let req = ::turbocharger::bincode::serialize(&#req {
     typetag_const_one: 1,
     dispatch_name: #orig_fn_string,
     txid: tx.txid,
     params: (#( #orig_fn_param_names ),* #orig_fn_params_maybe_comma),
    })
    .unwrap();
    tx.send_ws(req);
    let response = tx.resp().await;
    let #resp { result, .. } =
     ::turbocharger::bincode::deserialize(&response).unwrap();
    result
   }
  },
 };

 quote! {
  #[cfg(target_arch = "wasm32")]
  #[allow(unused_imports)]
  use wasm_bindgen::prelude::*;

  #[cfg(not(target_arch = "wasm32"))]
  #orig_fn

  #[cfg(not(target_arch = "wasm32"))]
  #[allow(non_snake_case)]
  mod #mod_name {
   use ::turbocharger::typetag;
   #[::turbocharger::typetag::serde(name = #orig_fn_string)]
   #[::turbocharger::async_trait]
   impl ::turbocharger::RPC for super::#dispatch {
    async fn execute(&self, sender: Box<dyn Fn(Vec<u8>) + Send>) {
     #executebody
    }
   }
  }

  #wasm_side

  #[cfg(not(target_arch = "wasm32"))]
  #[allow(non_snake_case)]
  async fn #remote_fn_ident(peer: &str, #orig_fn_params) -> #serialize_ret_ty {
   let tx = ::turbocharger::_Transaction::new();
   let req = ::turbocharger::bincode::serialize(&#req {
    typetag_const_one: 1,
    dispatch_name: #orig_fn_string,
    txid: tx.txid,
    params: (#( #orig_fn_param_names ),* #orig_fn_params_maybe_comma),
   })
   .unwrap();
   tx.send_udp(peer, req).await;
   let response = tx.resp().await;
   let #resp { result, .. } =
    ::turbocharger::bincode::deserialize(&response).unwrap();
   result
  }

  #[allow(non_camel_case_types)]
  #[derive(::turbocharger::serde::Serialize, ::turbocharger::serde::Deserialize)]
  #[serde(crate = "::turbocharger::serde")]
  struct #req {
   typetag_const_one: i64,
   dispatch_name: &'static str,
   txid: i64,
   params: (#( #orig_fn_param_tys ),* #orig_fn_params_maybe_comma),
  }

  #[allow(non_camel_case_types)]
  #[derive(::turbocharger::serde::Serialize, ::turbocharger::serde::Deserialize)]
  #[serde(crate = "::turbocharger::serde")]
  struct #dispatch {
   txid: i64,
   params: (#( #orig_fn_param_tys ),* #orig_fn_params_maybe_comma),
  }

  #[allow(non_camel_case_types)]
  #[derive(::turbocharger::serde::Serialize, ::turbocharger::serde::Deserialize)]
  #[serde(crate = "::turbocharger::serde")]
  struct #resp {
   txid: i64,
   result: #serialize_ret_ty,
  }
 }
}
