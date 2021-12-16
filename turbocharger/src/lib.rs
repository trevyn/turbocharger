#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

pub use turbocharger_impl::{backend, server_only, wasm_only};

#[cfg(not(target_arch = "wasm32"))]
#[doc(hidden)]
pub use async_trait::async_trait;
#[doc(hidden)]
pub use bincode;
use futures::{SinkExt, StreamExt};
#[cfg(target_arch = "wasm32")]
pub use js_sys;
#[doc(hidden)]
pub use serde;
#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, collections::HashMap};
#[cfg(not(target_arch = "wasm32"))]
#[doc(hidden)]
pub use typetag;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
#[doc(hidden)]
#[typetag::serde]
#[async_trait]
pub trait RPC: Send + Sync {
 async fn execute(&self) -> Vec<u8>;
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct Globals {
 channel_tx: Option<futures::channel::mpsc::UnboundedSender<Vec<u8>>>,
 next_txid: i64,
 senders: HashMap<i64, futures::channel::mpsc::UnboundedSender<Vec<u8>>>,
}

#[cfg(target_arch = "wasm32")]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Response {
 pub txid: i64,
 pub resp: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
thread_local! {
 static G: RefCell<Globals> = RefCell::new(Globals::default());
}

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! console_log {
  ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
 }

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
 #[wasm_bindgen(js_namespace = console)]
 fn log(s: &str);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn warp_routes<A: 'static + rust_embed::RustEmbed>(
 asset: A,
) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
 use warp::Filter;
 warp_socket_route().or(warp_rust_embed_route(asset)).boxed()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn warp_socket_route() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
 use warp::Filter;
 warp::path("turbocharger_socket")
  .and(warp::ws())
  .map(|ws: warp::ws::Ws| ws.on_upgrade(accept_connection))
  .boxed()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn warp_rust_embed_route<A: rust_embed::RustEmbed>(
 _asset: A,
) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
 use warp::Filter;
 warp::path::full()
  .map(|path: warp::path::FullPath| {
   let path = match path.as_str().trim_start_matches('/') {
    "" => "index.html",
    path => path,
   };
   match A::get(path) {
    None => warp::http::Response::builder().status(404).body("404 not found!".into()).unwrap(),
    Some(asset) => {
     let mime = mime_guess::from_path(path).first_or_octet_stream();
     let mut res = warp::reply::Response::new(asset.data.into());
     res
      .headers_mut()
      .insert("content-type", warp::http::header::HeaderValue::from_str(mime.as_ref()).unwrap());
     res
    }
   }
  })
  .boxed()
}

#[cfg(not(target_arch = "wasm32"))]
async fn accept_connection(ws: warp::ws::WebSocket) {
 use futures::TryFutureExt;

 log::debug!("accept_connection");

 let (mut ws_tx, mut ws_rx) = ws.split();
 let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
 let mut rx = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

 tokio::task::spawn(async move {
  while let Some(msg) = rx.next().await {
   ws_tx
    .send(msg)
    .unwrap_or_else(|e| {
     log::warn!("websocket send error: {}", e);
    })
    .await;
  }
 });

 while let Some(result) = ws_rx.next().await {
  let msg = match result {
   Ok(msg) => msg,
   Err(e) => {
    log::warn!("websocket error: {}", e);
    break;
   }
  };
  let tx_clone = tx.clone();
  tokio::task::spawn(async move {
   let msg = msg.as_bytes();
   if !msg.is_empty() {
    let target_func: Box<dyn RPC> = bincode::deserialize(msg).unwrap();
    let response = target_func.execute().await;
    tx_clone.send(warp::ws::Message::binary(response)).unwrap();
   }
  });
 }

 log::warn!("accept_connection completed")
}

#[cfg(target_arch = "wasm32")]
pub struct _Transaction {
 pub txid: i64,
 channel_tx: futures::channel::mpsc::UnboundedSender<Vec<u8>>,
 resp_rx: futures::channel::mpsc::UnboundedReceiver<Vec<u8>>,
}

#[cfg(target_arch = "wasm32")]
impl _Transaction {
 pub fn new() -> Self {
  let (resp_tx, resp_rx) = futures::channel::mpsc::unbounded();

  let (channel_tx, txid) = G.with(|g| -> (_, _) {
   let mut g = g.borrow_mut();
   let txid = g.next_txid;
   g.senders.insert(txid, resp_tx);
   g.next_txid += 1;
   (g.channel_tx.clone().unwrap(), txid)
  });

  _Transaction { txid, channel_tx, resp_rx }
 }

 pub async fn run(mut self, req: Vec<u8>) -> Vec<u8> {
  self.channel_tx.send(req).await.unwrap();
  self.resp_rx.next().await.unwrap()
 }
}

#[cfg(target_arch = "wasm32")]
impl Default for _Transaction {
 fn default() -> Self {
  Self::new()
 }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
 console_error_panic_hook::set_once();

 let (channel_tx, mut channel_rx) = futures::channel::mpsc::unbounded();

 G.with(|g| {
  g.borrow_mut().channel_tx = Some(channel_tx);
 });

 console_log!("connecting");

 let (_ws, wsio) =
  ws_stream_wasm::WsMeta::connect("ws://127.0.0.1:8080/turbocharger_socket", None).await.unwrap();

 console_log!("connected");

 let (mut ws_tx, mut ws_rx) = wsio.split();

 wasm_bindgen_futures::spawn_local(async move {
  while let Some(msg) = ws_rx.next().await {
   if let ws_stream_wasm::WsMessage::Binary(msg) = msg {
    let txid = i64::from_le_bytes(msg[0..8].try_into().unwrap());
    let mut sender = G.with(|g| -> _ { g.borrow().senders.get(&txid).unwrap().clone() });
    sender.send(msg).await.unwrap();
   }
  }
  console_log!("ws_rx ENDED");
 });

 wasm_bindgen_futures::spawn_local(async move {
  while let Some(msg) = channel_rx.next().await {
   ws_tx.send(ws_stream_wasm::WsMessage::Binary(msg)).await.unwrap();
  }
  console_log!("rx ENDED");
 });

 Ok(())
}
