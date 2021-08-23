#[allow(unused_imports)]
use wasm_bindgen::prelude::*;
//use turbocharger::prelude::*;
// use turbocharger::serde;

//#[wasm]
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn wasm_get_greeting() -> String {
 "Hello from WASM.".to_string()
}

//#[backend]
#[cfg(target_arch = "wasm32")]
pub async fn backend_get_greeting() -> String {
 {
  let retval: String = turbocharger::bincode::deserialize(
   ::turbocharger::_make_rpc_call("backend_get_greeting".to_string()).await.as_ref(),
  )
  .unwrap();
  retval
 }
}
#[cfg(not(target_arch = "wasm32"))]
pub async fn backend_get_greeting() -> String {
 eprintln!("backend_get_greeting called");
 "Hello from backend.".to_string()
}
#[cfg(not(target_arch = "wasm32"))]
mod _tc_backend_get_greeting {
 use ::turbocharger::typetag;
 #[::turbocharger::typetag::serde(name = "_tc_backend_get_greeting")]
 #[::turbocharger::async_trait]
 impl ::turbocharger::RPC for super::_tc_backend_get_greeting_return {
  async fn execute(&self) -> Vec<u8> {
   ::turbocharger::bincode::serialize(&super::backend_get_greeting().await).unwrap()
  }
 }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(non_camel_case_types)]
#[derive(::turbocharger::serde::Serialize, ::turbocharger::serde::Deserialize)]
#[serde(crate = "::turbocharger::serde")]
struct _tc_backend_get_greeting_return(String);

//#[server_only]
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
#[tokio::main]
async fn main() {
 eprintln!("{:?}", backend_get_greeting().await);
 let event: &dyn ::turbocharger::RPC = &_tc_backend_get_greeting_return("foo".to_string());
 let json = serde_json::to_string(&event).unwrap();
 println!("{}", json);
 eprintln!("deserializing...");

 let event: Box<dyn ::turbocharger::RPC> =
  serde_json::from_str(r#"{"_tc_backend_get_greeting":"foo"}"#).unwrap();
 dbg!(event.execute().await);

 eprintln!("Serving on http://127.0.0.1:8080");
 warp::serve(turbocharger::warp_routes()).run(([127, 0, 0, 1], 8080)).await;
}
