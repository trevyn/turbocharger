#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
//use turbocharger::prelude::*;
//use turbocharger::{wasm_only, backend, server_only};
#[cfg(target_arch = "wasm32")]
use turbocharger::console_log;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
 #[wasm_bindgen(js_namespace = console)]
 fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
#[allow(non_camel_case_types)]
#[wasm_bindgen]
pub struct wasm_only;

#[cfg(target_arch = "wasm32")]
#[allow(non_camel_case_types)]
#[wasm_bindgen]
pub struct backend;

//#[wasm_only]
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_class = wasm_only)]
impl wasm_only {
 #[wasm_bindgen]
 pub async fn get_local_greeting1() -> String {
  "Hello from WASM one.".to_string()
 }
}

//#[wasm_only]
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_class = wasm_only)]
impl wasm_only {
 #[wasm_bindgen]
 pub async fn get_local_greeting2() -> String {
  "Hello from WASM two.".to_string()
 }
}

//#[backend]
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_class = backend)]
impl backend {
 #[wasm_bindgen]
 pub async fn get_remote_greeting() -> String {
  {
   let result =
    ::turbocharger::_make_rpc_call(r#"{"_tc_get_remote_greeting":[]}"#.to_string()).await; //.as_ref();
   console_log!("{:?}", result);
   // let retval: String = turbocharger::bincode::deserialize(result).unwrap();
   "result".to_string() //result
  }
 }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_remote_greeting() -> String {
 eprintln!("get_remote_greeting called");
 "Hello from backend.".to_string()
}

#[cfg(not(target_arch = "wasm32"))]
mod _tc_get_remote_greeting {
 use ::turbocharger::typetag;
 #[::turbocharger::typetag::serde(name = "_tc_get_remote_greeting")]
 #[::turbocharger::async_trait]
 impl ::turbocharger::RPC for super::_tc_get_remote_greeting_params {
  async fn execute(&self) -> Vec<u8> {
   ::turbocharger::bincode::serialize(&super::get_remote_greeting().await).unwrap()
  }
 }
}

#[allow(non_camel_case_types)]
#[derive(::turbocharger::serde::Serialize, ::turbocharger::serde::Deserialize)]
#[serde(crate = "::turbocharger::serde")]
struct _tc_get_remote_greeting_params();

#[allow(non_camel_case_types)]
#[derive(::turbocharger::serde::Serialize, ::turbocharger::serde::Deserialize)]
#[serde(crate = "::turbocharger::serde")]
struct _tc_get_remote_greeting_result(String);

//#[server_only]
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
#[tokio::main]
async fn main() {
 eprintln!("{:?}", get_remote_greeting().await);
 let event: &dyn ::turbocharger::RPC = &_tc_get_remote_greeting_params();
 let json = serde_json::to_string(&event).unwrap();
 println!("{}", json);
 eprintln!("deserializing...");

 let event: Box<dyn ::turbocharger::RPC> =
  serde_json::from_str(r#"{"_tc_get_remote_greeting":[]}"#).unwrap();
 dbg!(event.execute().await);

 eprintln!("Serving on http://127.0.0.1:8080");
 warp::serve(turbocharger::warp_routes()).run(([127, 0, 0, 1], 8080)).await;
}
