#[cfg(not(target_arch = "wasm32"))]
pub use async_trait::async_trait;
pub use bincode;
use futures::{SinkExt, StreamExt};
pub use serde;
#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, collections::HashMap};
#[cfg(not(target_arch = "wasm32"))]
pub use typetag;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
#[typetag::serde]
#[async_trait]
pub trait RPC {
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
pub fn warp_routes() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
 use warp::Filter;
 warp::path("turbocharger_socket")
  .and(warp::ws())
  .map(|ws: warp::ws::Ws| ws.on_upgrade(accept_connection))
  .boxed()
}

#[cfg(not(target_arch = "wasm32"))]
async fn accept_connection(ws: warp::ws::WebSocket) {
 use futures::{SinkExt, StreamExt, TryFutureExt};

 eprintln!("accept_connection");

 let (mut ws_tx, mut ws_rx) = ws.split();
 let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
 let mut rx = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

 tokio::task::spawn(async move {
  while let Some(msg) = rx.next().await {
   ws_tx
    .send(msg)
    .unwrap_or_else(|e| {
     eprintln!("websocket send error: {}", e);
    })
    .await;
  }
 });

 while let Some(result) = ws_rx.next().await {
  let msg = match result {
   Ok(msg) => msg,
   Err(e) => {
    eprintln!("websocket error: {}", e);
    break;
   }
  };

  eprintln!("got msg: {:?}", msg);

  // let WrappedCommand { txid, cmd } = bincode::deserialize(msg.as_bytes()).unwrap();
  // eprintln!("txid {}: {:?}", txid, cmd);
 }

 eprintln!("accept_connection completed")
}

#[cfg(target_arch = "wasm32")]
pub async fn _make_rpc_call(s: String) -> Vec<u8> {
 console_log!("_make_rpc_call: {:?}", s);

 let (resp_tx, mut resp_rx) = futures::channel::mpsc::unbounded();

 let (mut channel_tx, txid) = G.with(|g| -> (_, _) {
  let mut g = g.borrow_mut();
  let txid = g.next_txid;
  g.senders.insert(txid, resp_tx);
  g.next_txid += 1;
  (g.channel_tx.clone().unwrap(), txid)
 });

 channel_tx.send(s.into_bytes()).await.unwrap();
 // let wrapped_cmd = WrappedCommand { txid, cmd };
 // channel_tx.send(bincode::serialize(&wrapped_cmd).unwrap()).await.unwrap();
 // resp_rx.next().await.unwrap()

 vec![42]
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
    let Response { txid, resp } = bincode::deserialize(&msg).unwrap();
    let mut sender = G.with(|g| -> _ { g.borrow().senders.get(&txid).unwrap().clone() });
    sender.send(resp).await.unwrap();
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
