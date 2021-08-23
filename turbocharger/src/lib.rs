#[cfg(not(target_arch = "wasm32"))]
pub use async_trait::async_trait;
pub use bincode;
pub use serde;
#[cfg(not(target_arch = "wasm32"))]
pub use typetag;
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
#[typetag::serde]
#[async_trait]
pub trait RPC {
 async fn execute(&self) -> Vec<u8>;
}

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! console_log {
  ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
 }

#[wasm_bindgen]
extern "C" {
 #[wasm_bindgen(js_namespace = console)]
 fn log(s: &str);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn warp_routes() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
 use warp::Filter;
 warp::path("turbocharger_socket")
  .and(warp::ws())
  .map(|ws: warp::ws::Ws| ws.on_upgrade(accept_connection))
  .boxed()
}

#[cfg(not(target_arch = "wasm32"))]
async fn accept_connection(_ws: warp::ws::WebSocket) {}

#[cfg(target_arch = "wasm32")]
pub async fn _make_rpc_call(s: String) -> Vec<u8> {
 vec![]
}
