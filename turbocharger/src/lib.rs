#![deny(unsafe_code)]
#![doc = include_str!("../README.md")]

use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub use turbocharger_impl::{backend, server_only, wasm_only};

#[wasm_only]
#[cfg(feature = "svelte")]
#[wasm_bindgen(typescript_custom_section)]
const Subscriber: &'static str = r#"
import { Subscriber } from "svelte/store";
"#;

#[doc(hidden)]
pub use {bincode, futures_util, serde};

#[server_only]
#[doc(hidden)]
pub use {async_stream, async_trait::async_trait, stream_cancel, typetag};

#[cfg(target_arch = "wasm32")]
#[doc(hidden)]
pub use js_sys;

#[server_only]
#[doc(hidden)]
#[typetag::serde]
#[async_trait]
pub trait RPC: Send + Sync {
 async fn execute(
  &self,
  sender: Box<dyn Fn(Vec<u8>) + Send>,
  tripwire: Option<stream_cancel::Tripwire>,
  remote_addr: Option<std::net::SocketAddr>,
  user_agent: Option<String>,
 );
 fn txid(&self) -> i64;
}

struct Globals {
 #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
 channel_tx: Option<futures_channel::mpsc::UnboundedSender<Vec<u8>>>,
 #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
 socket_url: Option<String>,
 next_txid: i64,
 senders: std::collections::HashMap<i64, futures_channel::mpsc::UnboundedSender<Vec<u8>>>,
}

impl Default for Globals {
 fn default() -> Self {
  Self { socket_url: None, channel_tx: None, next_txid: 256, senders: Default::default() }
 }
}

#[doc(hidden)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Response {
 pub txid: i64,
 pub resp: Vec<u8>,
}

static G: Lazy<Mutex<Globals>> = Lazy::new(Mutex::default);

#[server_only]
static UDP_SOCKET: Lazy<Mutex<Option<std::sync::Arc<tokio::net::UdpSocket>>>> =
 Lazy::new(Mutex::default);

#[wasm_only]
#[macro_export]
macro_rules! console_log {
 ($($t:tt)*) => (
  ::turbocharger::call_console_log(&format_args!($($t)*).to_string());
 )
}

#[wasm_only]
macro_rules! tc_console_log {
 ($($t:tt)*) => (
  call_console_log(&format_args!($($t)*).to_string());
 )
}

#[wasm_only]
#[doc(hidden)]
pub fn call_console_log(msg: &str) {
 #[allow(unsafe_code)]
 #[allow(unused_unsafe)]
 unsafe {
  log(msg);
 }
}

#[wasm_only]
#[wasm_bindgen]
extern "C" {
 #[wasm_bindgen(js_namespace = console)]
 fn log(s: &str);
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "axum_server")]
mod axum_server;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "axum_server")]
pub use axum_server::serve;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(all(feature = "tls", feature = "axum_server"))]
pub use axum_server::serve_tls;

#[doc(hidden)]
pub struct _Transaction {
 pub txid: i64,
 resp_rx: futures_channel::mpsc::UnboundedReceiver<Vec<u8>>,
}

impl _Transaction {
 pub fn new() -> Self {
  let (resp_tx, resp_rx) = futures_channel::mpsc::unbounded();

  let mut g = G.lock().unwrap();

  let txid = g.next_txid;
  g.senders.insert(txid, resp_tx);
  g.next_txid += 1;

  _Transaction { txid, resp_rx }
 }

 #[cfg(target_arch = "wasm32")]
 pub fn send_ws(&self, req: Vec<u8>) {
  wasm_bindgen_futures::spawn_local(async move {
   ensure_ws_connected().await;
   let mut channel = G.lock().unwrap().channel_tx.clone().unwrap();
   channel.send(req).await.unwrap();
  });
 }

 #[server_only]
 pub async fn send_udp(&self, peer: &str, req: Vec<u8>) {
  let socket = UDP_SOCKET.lock().unwrap().clone().unwrap();
  socket.send_to(&req, peer).await.unwrap();
 }

 pub async fn resp(mut self) -> Vec<u8> {
  self.resp_rx.next().await.unwrap()
 }

 #[cfg(target_arch = "wasm32")]
 pub fn set_sender(mut self, sender: Box<dyn Fn(Vec<u8>)>) {
  wasm_bindgen_futures::spawn_local(async move {
   while let Some(msg) = self.resp_rx.next().await {
    sender(msg);
   }
  });
 }
}

impl Default for _Transaction {
 fn default() -> Self {
  Self::new()
 }
}

/// _Experimental._ Spawns a new Turbocharger UDP server. Future resolves when the server is ready to respond to requests.
#[server_only]
#[tracked::tracked]
#[doc(hidden)]
pub async fn spawn_udp_server(port: u16) -> tracked::Result<()> {
 let socket = std::sync::Arc::new(tokio::net::UdpSocket::bind(format!("0.0.0.0:{}", port)).await?);
 log::debug!("Listening on: {}", socket.local_addr()?);
 *UDP_SOCKET.lock().unwrap() = Some(socket.clone());

 tokio::spawn(async move {
  loop {
   let mut buf = [0; 1500];
   let (size, peer) = socket.recv_from(&mut buf).await.unwrap();
   log::debug!("received {} bytes from {}", size, peer);
   if size < 8 {
    continue;
   };
   let first_word = i64::from_le_bytes(buf[0..8].try_into().unwrap());
   let msg = buf[0..size].to_vec();
   match first_word {
    1 => {
     // typetagged request
     let send_socket = socket.clone();
     tokio::task::spawn(async move {
      let target_func: Box<dyn RPC> = bincode::deserialize(&msg).unwrap();
      let sender = Box::new(move |response: Vec<u8>| {
       let send_socket_cloned = send_socket.clone();
       tokio::task::spawn(async move {
        send_socket_cloned.send_to(&response, peer).await.unwrap();
       });
      });
      target_func.execute(sender, None, Some(peer), Some("udp".into())).await;
     });
    }
    txid => {
     // response txid
     let mut sender = G.lock().unwrap().senders.get(&txid).unwrap().clone();
     sender.send(msg).await.unwrap();
    }
   }
  }
 });

 Ok(())
}

#[wasm_only]
#[wasm_bindgen]
pub fn set_socket_url(url: String) {
 let mut g = G.lock().unwrap();
 g.socket_url = Some(url);
}

#[wasm_only]
async fn ensure_ws_connected() {
 let mut g = G.lock().unwrap();

 if g.socket_url.is_none() {
  g.socket_url = Some({
   let location = web_sys::window().unwrap().location();
   let protocol = match location.protocol().unwrap().as_str() {
    "https:" => "wss:",
    _ => "ws:",
   };
   format!("{}//{}/turbocharger_socket", protocol, location.host().unwrap())
  });
 }
 let socket_url = g.socket_url.clone().unwrap();

 if g.channel_tx.is_none() {
  let (channel_tx, mut channel_rx) = futures_channel::mpsc::unbounded();
  g.channel_tx = Some(channel_tx);

  drop(g);

  tc_console_log!("connecting to {}", socket_url);

  let (_ws, wsio) = ws_stream_wasm::WsMeta::connect(socket_url, None).await.unwrap();

  tc_console_log!("connected");

  let (mut ws_tx, mut ws_rx) = wsio.split();

  wasm_bindgen_futures::spawn_local(async move {
   while let Some(msg) = ws_rx.next().await {
    if let ws_stream_wasm::WsMessage::Binary(msg) = msg {
     let txid = i64::from_le_bytes(msg[0..8].try_into().unwrap());
     let mut sender = G.lock().unwrap().senders.get(&txid).unwrap().clone();
     sender.send(msg).await.unwrap();
    }
   }
   tc_console_log!("ws_rx ENDED");
  });

  wasm_bindgen_futures::spawn_local(async move {
   while let Some(msg) = channel_rx.next().await {
    ws_tx.send(ws_stream_wasm::WsMessage::Binary(msg)).await.unwrap();
   }
   tc_console_log!("rx ENDED");
  });
 }
}

#[wasm_only]
#[cfg(feature = "console_error_panic_hook")]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
 console_error_panic_hook::set_once();
 Ok(())
}
