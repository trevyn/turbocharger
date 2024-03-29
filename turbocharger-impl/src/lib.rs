//! This crate provides Turbocharger's procedural macros.
//!
//! Please refer to the `turbocharger` crate for details.

#![forbid(unsafe_code)]

mod extract;

use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, parse_quote, spanned::Spanned};

#[proc_macro]
pub fn remote_addr(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
	quote!(_turbocharger_connection_info.as_ref().and_then(|ref i| i.remote_addr)).into()
}

#[proc_macro]
pub fn user_agent(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
	quote!(_turbocharger_connection_info.as_ref().and_then(|ref i| i.user_agent.as_ref())).into()
}

struct ConnectionLocal {
	pub ident: syn::Ident,
	pub ty: syn::Type,
}

impl syn::parse::Parse for ConnectionLocal {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let ident = input.parse()?;

		let _: syn::Token![:] = input.parse()?;
		let _: syn::Token![&] = input.parse()?;
		let _: syn::Token![mut] = input.parse()?;

		let ty = input.parse()?;
		Ok(Self { ident, ty })
	}
}

#[proc_macro]
pub fn connection_local(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let ConnectionLocal { ident, ty } = parse_macro_input!(input as ConnectionLocal);
	let ident_str = ident.to_string();

	quote! {
		let mut _turbocharger_connection_local_map = match _turbocharger_connection_info.as_ref() {
			Some(c) => Some(c.connection_local.lock().await),
			None => None,
		};

		let #ident = {
			_turbocharger_connection_local_map
				.as_mut()
				.unwrap()
				.entry(( #ident_str , std::any::TypeId::of::< #ty >()))
				.or_insert_with(|| Box::new( #ty ::default()))
				.downcast_mut::< #ty >()
				.unwrap()
		};
	}
	.into()
}

/// Apply this to an item to make it available on the server target only.
///
/// Only adds `#[cfg(not(target_arch = "wasm32"))]`
#[proc_macro_attribute]
pub fn server_only(
	_args: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	let orig_item = parse_macro_input!(input as syn::Item);
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
	let orig_item = parse_macro_input!(input as syn::Item);
	proc_macro::TokenStream::from(quote! {
		#[cfg(any(feature = "wasm", target_arch = "wasm32"))]
		#[allow(unused_imports)]
		use wasm_bindgen::prelude::*;

		#[cfg(any(feature = "wasm", target_arch = "wasm32"))]
		#[allow(clippy::bind_instead_of_map)]
		#orig_item
	})
}

/// Apply this to a `pub async fn` to make it available (over the network) to the frontend. Also apply to any `struct`s used in backend function signatures. If the fn will be called from a JavaScript frontend, use `#[backend(js)]`.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn backend(
	args: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	backend_item(args, syn::parse_macro_input!(input as syn::Item)).into()
}

fn backend_item(args: proc_macro::TokenStream, orig_item: syn::Item) -> proc_macro2::TokenStream {
	match orig_item {
		syn::Item::Fn(orig) => backend_fn(args, orig),
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
	if !is_rust_analyzer() {
		let mut api_struct = orig_struct.clone();
		api_struct.vis = parse_quote!();
		api_struct.attrs.retain(|attr| attr.path().is_ident("doc"));
		for field in &mut api_struct.fields {
			field.vis = parse_quote!();
			field.attrs.retain(|attr| attr.path().is_ident("doc"));
		}

		let lockfile = std::fs::File::create(std::env::temp_dir().join("turbocharger.lock")).unwrap();
		fs2::FileExt::lock_exclusive(&lockfile).unwrap();

		// add api_struct to file if it doesn't already exist, or replace it if it does
		let mut file = read_backend_api_rs();

		let mut found = false;
		for item in &mut file.items {
			if let syn::Item::Struct(ref mut item) = item {
				if item.ident == api_struct.ident {
					found = true;
					*item = api_struct.clone();
					break;
				}
			}
		}
		if !found {
			file.items.push(syn::Item::Struct(api_struct));
		}

		write_backend_api_rs(file);
	}

	#[allow(clippy::redundant_clone)]
	let syn::ItemStruct { attrs, ident, fields, .. } = orig_struct.clone();

	let output = quote! {
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
	};

	std::fs::create_dir_all("target/generated/turbocharger").ok();
	std::fs::write(
		project_root_path_with(format!("target/generated/turbocharger/{}.rs", orig_struct.ident)),
		prettyplease::unparse(&parse_quote!( #output )),
	)
	.unwrap();

	// std::process::Command::new("rustfmt")
	//  .current_dir(project_root_path_with("debug_expansions"))
	//  .arg("--edition")
	//  .arg("2021")
	//  .arg("--config")
	//  .arg("tab_spaces=1")
	//  .arg(format!("{}.rs", orig_struct.ident))
	//  .output()
	//  .unwrap();

	output
}

fn backend_fn(args: proc_macro::TokenStream, orig_fn: syn::ItemFn) -> proc_macro2::TokenStream {
	let is_js = args.to_string() == "js";

	if !is_rust_analyzer() {
		let mut api_fn = orig_fn.clone();
		api_fn.vis = parse_quote!();
		api_fn.attrs.retain(|attr| attr.path().is_ident("doc"));
		api_fn.block = parse_quote!({});

		let lockfile = std::fs::File::create(std::env::temp_dir().join("turbocharger.lock")).unwrap();
		fs2::FileExt::lock_exclusive(&lockfile).unwrap();

		// add api_fn to file if it doesn't already exist, or replace it if it does
		let mut file = read_backend_api_rs();

		let mut found = false;
		for item in &mut file.items {
			if let syn::Item::Fn(ref mut item) = item {
				if item.sig.ident == api_fn.sig.ident {
					found = true;
					*item = api_fn.clone();
					break;
				}
			}
		}
		if !found {
			file.items.push(syn::Item::Fn(api_fn));
		}

		write_backend_api_rs(file);
	}

	let orig_fn_ident = orig_fn.sig.ident.clone();
	let orig_fn_string = orig_fn_ident.to_string();
	let orig_fn_params = orig_fn.sig.inputs.clone();
	let orig_fn_stmts = &orig_fn.block.stmts;
	let orig_fn_stmts = quote!(#( #orig_fn_stmts )*);

	let store_name = format_ident!("_TURBOCHARGER_STORE_{}", orig_fn_ident);
	let dispatch = format_ident!("_TURBOCHARGER_DISPATCH_{}", orig_fn_ident);
	let req = format_ident!("_TURBOCHARGER_REQ_{}", orig_fn_ident);
	let resp = format_ident!("_TURBOCHARGER_RESP_{}", orig_fn_ident);
	let js_fn_ident = format_ident!("_TURBOCHARGER_JS_{}", orig_fn_ident);
	let remote_fn_ident = format_ident!("remote_{}", orig_fn_ident);
	let remote_impl_ident = format_ident!("_TURBOCHARGER_REMOTEIMPL_{}", orig_fn_ident);
	let subscriber_fn_ident = format_ident!("_TURBOCHARGER_SUBSCRIBERFN_{}", orig_fn_ident);

	let orig_fn_ret_ty = match orig_fn.sig.output.clone() {
		syn::ReturnType::Type(_, path) => *path,
		syn::ReturnType::Default => parse_quote! { () },
	};
	let stream_inner_ty = extract::extract_stream(&orig_fn_ret_ty);
	if stream_inner_ty.is_some() && orig_fn.sig.asyncness.is_some() {
		abort!(orig_fn.sig.asyncness, "#[backend] functions that return a stream must not be async."; help = "Remove the `async`."
		);
	}
	let result_inner_ty = extract::extract_result(stream_inner_ty.unwrap_or(&orig_fn_ret_ty));
	let store_value_ty = if result_inner_ty.is_some() {
		quote! { Result<#result_inner_ty, JsValue> }
	} else {
		quote! { #stream_inner_ty }
	};

	let maybe_map_err_jsvalue = match result_inner_ty {
		Some(_) => quote! { .map_err(|e| ::turbocharger::js_sys::Error::new(&e.to_string()).into()) },
		None => quote! {},
	};

	let send_value_to_subscription = if result_inner_ty.is_some() {
		quote! {
			if let Some(value) = self.value.lock().unwrap().clone() {
				let promise: ::turbocharger::js_sys::Promise = match value.clone() {
					Ok(t) => ::turbocharger::js_sys::Promise::resolve(&t.into()).into(),
					Err(e) => ::turbocharger::js_sys::Promise::reject(&e.into()).into(),
				};
				subscription.call1(&JsValue::null(), &promise).ok();
			}
		}
	} else {
		quote! {
			if let Some(value) = self.value.lock().unwrap().clone() {
				subscription.call1(&JsValue::null(), &value.into()).ok();
			}
		}
	};

	let send_value_to_subscriptions = if result_inner_ty.is_some() {
		quote! {
			if let Some(value) = value.lock().unwrap().clone() {
				let promise: ::turbocharger::js_sys::Promise = match value.clone() {
					Ok(t) => ::turbocharger::js_sys::Promise::resolve(&t.into()).into(),
					Err(e) => ::turbocharger::js_sys::Promise::reject(&e.into()).into(),
				};
				for subscription in subscriptions.lock().unwrap().iter() {
					if let Some(subscription) = subscription.lock().unwrap().as_ref() {
						subscription.call1(&JsValue::null(), &promise).ok();
					}
				}
			}
		}
	} else {
		quote! {
			if let Some(value) = value.lock().unwrap().clone() {
				for subscription in subscriptions.lock().unwrap().iter() {
					if let Some(subscription) = subscription.lock().unwrap().as_ref() {
						subscription.call1(&JsValue::null(), &value.clone().into()).ok();
					}
				}
			}
		}
	};

	let bindgen_ret_ty = match (stream_inner_ty, result_inner_ty) {
		(None, Some(ty)) => quote! { Result<#ty, JsValue> },
		(Some(_ty), _) => quote! { #store_name },
		(None, None) => quote! { #orig_fn_ret_ty },
	};
	let serialize_ret_ty = match &stream_inner_ty {
		Some(ty) => quote! { #ty },
		None => quote! { #orig_fn_ret_ty },
	};

	let orig_fn_ret_ty = if let Some(ty) = stream_inner_ty {
		quote_spanned! {ty.span()=> impl ::turbocharger::futures_util::stream::Stream<Item = #ty > }
	} else {
		quote_spanned! {orig_fn_ret_ty.span()=> #orig_fn_ret_ty }
	};

	let tuple_indexes = (0..orig_fn_params.len()).map(syn::Index::from);
	let orig_fn_param_names: Vec<_> = orig_fn_params
		.iter()
		.map(|p| match p {
			syn::FnArg::Receiver(_) => abort!(p, "I don't know what to do with `self` here."),
			syn::FnArg::Typed(pattype) => match *pattype.pat.clone() {
				syn::Pat::Ident(i) => i.ident,
				_ => abort!(pattype, "Parameter name is not Ident: {:?}", pattype),
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

	let orig_fn_params_maybe_comma = if orig_fn_params.is_empty() { quote!() } else { quote!( , ) };

	let mut orig_fn = orig_fn;
	orig_fn.sig.output = parse_quote! { -> #orig_fn_ret_ty };
	orig_fn.block = parse_quote!({
		let _turbocharger_connection_info: Option<::turbocharger::ConnectionInfo> = None;
		#orig_fn_stmts
	});

	let mut remote_impl_fn = orig_fn.clone();
	remote_impl_fn.sig.ident = remote_impl_ident.clone();
	remote_impl_fn.sig.inputs = parse_quote!(
		_turbocharger_connection_info: Option<::turbocharger::ConnectionInfo>
		#orig_fn_params_maybe_comma
		#orig_fn_params
	);
	remote_impl_fn.block = parse_quote!({ #orig_fn_stmts });

	let executebody = match &stream_inner_ty {
		Some(_ty) => quote! {
			use ::turbocharger::futures_util::stream::StreamExt as _;
			use ::turbocharger::stream_cancel::StreamExt as _;
			let stream = #remote_impl_ident(_turbocharger_connection_info #orig_fn_params_maybe_comma #( self.params. #tuple_indexes .clone() ),*);
			::turbocharger::futures_util::pin_mut!(stream);

			if let Some(tripwire) = tripwire {
				let mut incoming = stream.take_until_if(tripwire);
				while let Some(result) = incoming.next().await {
					let response = #resp {
						txid: self.txid,
						result: result.clone()
					};
					sender(::turbocharger::bincode::serialize(&response).unwrap());
				}
			}
			else {
				while let Some(result) = stream.next().await {
					let response = #resp {
						txid: self.txid,
						result: result.clone()
					};
					sender(::turbocharger::bincode::serialize(&response).unwrap());
				}
			}
		},
		None => quote! {
			let result = #remote_impl_ident(_turbocharger_connection_info #orig_fn_params_maybe_comma #( self.params. #tuple_indexes .clone() ),*).await;
			let response = #resp {
				txid: self.txid,
				result
			};
			sender(::turbocharger::bincode::serialize(&response).unwrap());
		},
	};

	let maybe_svelte_typescript_type = if cfg!(feature = "svelte") {
		quote! {
			#[cfg(target_arch = "wasm32")]
			#[wasm_bindgen]
			extern "C" {
				#[wasm_bindgen(typescript_type = "Subscriber<any>")]
				#[allow(non_camel_case_types)]
				pub type #subscriber_fn_ident;
			}
		}
	} else {
		quote!()
	};

	let wasm_side = match &stream_inner_ty {
		Some(_ty) => quote! {
			#[cfg(target_arch = "wasm32")]
			pub fn #orig_fn_ident(#orig_fn_params) -> #orig_fn_ret_ty {
				let tx = ::turbocharger::_Transaction::new();
				let req = ::turbocharger::bincode::serialize(&#req {
					typetag_const_one: 1,
					dispatch_name: #orig_fn_string,
					txid: tx.txid,
					params: (#( #orig_fn_param_names ),* #orig_fn_params_maybe_comma),
				})
				.unwrap();
				tx.send_ws(req);

				let (resp_tx, resp_rx) = ::turbocharger::futures_channel::mpsc::unbounded();

				tx.set_sender(Box::new(move |response| {
					use ::turbocharger::futures_util::SinkExt as _;
					let #resp { result, .. } =
						::turbocharger::bincode::deserialize(&response).unwrap();
					let mut resp_tx = resp_tx.clone();
					wasm_bindgen_futures::spawn_local(async move {
						resp_tx.send(result).await.unwrap();
					});
				}));

				resp_rx
			}
		},
		None => quote! {
			#[cfg(target_arch = "wasm32")]
			pub async fn #orig_fn_ident(#orig_fn_params) -> #orig_fn_ret_ty {
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

	let js_side = if !is_js {
		quote!()
	} else {
		match &stream_inner_ty {
			Some(_ty) => quote! {
				#[cfg(target_arch = "wasm32")]
				#[allow(non_camel_case_types)]
				#[wasm_bindgen]
				pub struct #store_name {
					req: std::sync::Arc<std::sync::Mutex<#req>>,
					value: std::sync::Arc<std::sync::Mutex<Option< #store_value_ty >>>,
					subscriptions: std::sync::Arc<std::sync::Mutex<Vec<std::sync::Arc<std::sync::Mutex<Option<::turbocharger::js_sys::Function>>>>>>,
				}

				#maybe_svelte_typescript_type

				#[cfg(target_arch = "wasm32")]
				#[wasm_bindgen]
				impl #store_name {
					#[wasm_bindgen]
					pub fn subscribe(&mut self, subscription: #subscriber_fn_ident) -> JsValue {
						let subscription: ::turbocharger::js_sys::Function = JsValue::from(subscription).into();
						if self.subscriptions.lock().unwrap().is_empty() {
							let tx = ::turbocharger::_Transaction::new();
							self.req.lock().unwrap().txid = tx.txid;
							tx.send_ws(::turbocharger::bincode::serialize(&*self.req.lock().unwrap()).unwrap());
							let subscriptions = self.subscriptions.clone();
							let value = self.value.clone();
							tx.set_sender(Box::new(move |response| {
								let #resp { result, .. } =
									::turbocharger::bincode::deserialize(&response).unwrap();
								value.lock().unwrap().replace(result.clone() #maybe_map_err_jsvalue );
								#send_value_to_subscriptions
							}));
						}

						#send_value_to_subscription
						let subscription_handle = std::sync::Arc::new(std::sync::Mutex::new(Some(subscription)));
						self.subscriptions.lock().unwrap().push(subscription_handle.clone());
						let subscriptions = self.subscriptions.clone();
						let req_clone = self.req.clone();

						Closure::wrap(Box::new(move || {
							subscription_handle.lock().unwrap().take();
							subscriptions.lock().unwrap().retain(|s| { s.lock().unwrap().is_some() });
							if subscriptions.lock().unwrap().is_empty() {
								let tx = ::turbocharger::_Transaction::new();
								tx.send_ws(::turbocharger::bincode::serialize(&*req_clone.lock().unwrap()).unwrap());
							}
						}) as Box<dyn Fn()>)
						.into_js_value()
					}
				}

				#[cfg(target_arch = "wasm32")]
				#[allow(non_snake_case)]
				#[wasm_bindgen(js_name = #orig_fn_ident)]
				pub fn #js_fn_ident(#orig_fn_params) -> #bindgen_ret_ty {
					let req = #req {
						typetag_const_one: 1,
						dispatch_name: #orig_fn_string,
						txid: 1,
						params: (#( #orig_fn_param_names ),* #orig_fn_params_maybe_comma),
					};
					#store_name {
						req: std::sync::Arc::new(std::sync::Mutex::new(req)),
						value: Default::default(),
						subscriptions: Default::default()
					}
				}
			},
			None => quote! {
				#[cfg(target_arch = "wasm32")]
				#[allow(non_snake_case)]
				#[wasm_bindgen(js_name = #orig_fn_ident)]
				pub async fn #js_fn_ident(#orig_fn_params) -> #bindgen_ret_ty {
					#orig_fn_ident(#( #orig_fn_param_names ),*) .await #maybe_map_err_jsvalue
				}
			},
		}
	};

	let output = quote! {
		#[cfg(target_arch = "wasm32")]
		#[allow(unused_imports)]
		use wasm_bindgen::prelude::*;

		#[cfg(not(target_arch = "wasm32"))]
		#[allow(unused_imports)]
		use turbocharger::prelude::*;

		#[cfg(not(target_arch = "wasm32"))]
		#[allow(dead_code)]
		#[tracked]
		#orig_fn

		#[cfg(not(target_arch = "wasm32"))]
		#[allow(non_snake_case)]
		#[tracked]
		#remote_impl_fn

		#[cfg(not(target_arch = "wasm32"))]
		#[allow(non_snake_case)]
		#[::turbocharger::typetag::serde(name = #orig_fn_string)]
		#[::turbocharger::async_trait]
		impl ::turbocharger::RPC for #dispatch {
			async fn execute(
				&self,
				sender: Box<dyn Fn(Vec<u8>) + Send>,
				tripwire: Option<::turbocharger::stream_cancel::Tripwire>,
				_turbocharger_connection_info: Option<::turbocharger::ConnectionInfo>
			) {
				#executebody
			}
			fn txid(&self) -> i64 {
				self.txid
			}
		}

		#wasm_side
		#js_side

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
	};

	std::fs::create_dir_all("target/generated/turbocharger").ok();
	std::fs::write(
		project_root_path_with(format!("target/generated/turbocharger/{}.rs", orig_fn_string)),
		prettyplease::unparse(&parse_quote!( #output )),
	)
	.unwrap();

	// std::process::Command::new("rustfmt")
	//  .current_dir(project_root_path_with("debug_expansions"))
	//  .arg("--edition")
	//  .arg("2021")
	//  .arg("--config")
	//  .arg("tab_spaces=1")
	//  .arg(format!("{}.rs", orig_fn_string))
	//  .output()
	//  .unwrap();

	output
}

fn read_backend_api_rs() -> syn::File {
	syn::parse_file(&std::fs::read_to_string(backend_api_rs_path()).unwrap_or_default()).unwrap()
}

fn write_backend_api_rs(file: syn::File) {
	let mut output = "// This file is auto-generated by Turbocharger.\n// Check it into version control to track API changes over time.\n// To regenerate: \"cargo clean && rm backend_api.rs && cargo check\"\n".to_string();
	let mut items = file.items;
	items.sort_by_key(|item| match item {
		syn::Item::Struct(s) => s.ident.to_string().to_lowercase(),
		syn::Item::Fn(f) => f.sig.ident.to_string().to_lowercase(),
		_ => unreachable!(),
	});
	for item in items {
		output.push('\n');
		output.push_str(&prettyplease::unparse(&parse_quote!( #item )));
	}
	if output != std::fs::read_to_string(backend_api_rs_path()).unwrap_or_default() {
		std::fs::write(backend_api_rs_path(), output).unwrap();
	}
}

fn backend_api_rs_path() -> std::path::PathBuf {
	project_root_path_with("backend_api.rs")
}

fn project_root_path_with<P: AsRef<std::path::Path>>(pushpath: P) -> std::path::PathBuf {
	let mut path = std::path::PathBuf::from(env!("OUT_DIR"));
	while path.file_name() != Some(std::ffi::OsStr::new("target")) {
		path.pop();
	}
	path.pop();
	path.push(pushpath);
	path
}

fn is_rust_analyzer() -> bool {
	std::env::current_exe()
		.unwrap()
		.file_stem()
		.unwrap()
		.to_string_lossy()
		.starts_with("rust-analyzer")
}
