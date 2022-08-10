#[cfg(feature = "tls")]
mod tls;

use axum::{
 body::{boxed, Full},
 extract::{
  ws::{Message, WebSocket, WebSocketUpgrade},
  ConnectInfo, TypedHeader,
 },
 handler::Handler,
 headers,
 http::{header, header::HeaderMap, StatusCode, Uri},
 response::{IntoResponse, Response},
 routing::{get, Router},
 Server,
};
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use rust_embed::RustEmbed;
use std::{
 collections::HashMap,
 marker::PhantomData,
 net::SocketAddr,
 sync::{Arc, Mutex},
};

/// Convenience function to run a full server with static files from `rust_embed` and the Turbocharger WebSocket.
pub async fn serve<A: 'static + RustEmbed>(addr: &SocketAddr) {
 let app = Router::new()
  .route("/turbocharger_socket", get(ws_handler))
  .fallback(rust_embed_handler::<A>.into_service());

 Server::bind(addr).serve(app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

/// Convenience function to run a full server with static files from `rust_embed` and the Turbocharger WebSocket.
#[cfg(feature = "tls")]
pub async fn serve_tls<A: 'static + RustEmbed>(addr: &SocketAddr) {
 let app = Router::new()
  .route("/turbocharger_socket", get(ws_handler))
  .fallback(rust_embed_handler::<A>.into_service());

 tls::serve(addr, app).await.unwrap();
}

/// Axum handler for serving static files from rust_embed.
pub async fn rust_embed_handler<A: RustEmbed>(uri: Uri, headers: HeaderMap) -> impl IntoResponse {
 if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
  if Some(if_none_match.to_str().unwrap_or_default()) == option_env!("BUILD_ID") {
   return StatusCode::NOT_MODIFIED.into_response();
  }
 }

 let mut path = uri.path().trim_start_matches('/').to_string();
 let is_brotli = headers
  .get(header::ACCEPT_ENCODING)
  .map(|enc| enc.to_str().unwrap_or_default().contains("br"))
  .unwrap_or(false);
 if path.is_empty() {
  path = "index.html".to_string();
 }
 StaticFile::<_, A> { path, is_brotli, phantomdata: PhantomData }.into_response()
}

struct StaticFile<T, A> {
 pub path: T,
 is_brotli: bool,
 phantomdata: PhantomData<A>,
}

impl<T, A> IntoResponse for StaticFile<T, A>
where
 T: Into<String>,
 A: RustEmbed,
{
 fn into_response(self) -> Response {
  let path = self.path.into();

  let content =
   A::get(format!("{}{}", path.as_str(), if self.is_brotli { ".br" } else { "" }).as_str());

  let (content, is_brotli) = if self.is_brotli && content.is_none() {
   (A::get(path.as_str()), false)
  } else {
   (content, self.is_brotli)
  };

  match content {
   Some(content) => {
    let body = boxed(Full::from(content.data));
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let resp = Response::builder().header(header::CONTENT_TYPE, mime.as_ref());
    let resp = if is_brotli { resp.header(header::CONTENT_ENCODING, "br") } else { resp };
    let resp = if let Some(build_id) = option_env!("BUILD_ID") {
     resp.header(header::ETAG, build_id)
    } else {
     resp
    };
    resp.body(body).unwrap()
   }
   None => {
    Response::builder().status(StatusCode::NOT_FOUND).body(boxed(Full::from("404"))).unwrap()
   }
  }
 }
}

/// Axum handler for serving the Turbocharger WebSocket.
pub async fn ws_handler(
 ws: WebSocketUpgrade,
 user_agent: Option<TypedHeader<headers::UserAgent>>,
 ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
 let ua_str =
  if let Some(TypedHeader(ua)) = user_agent { ua.as_str().into() } else { String::new() };

 ws.on_upgrade(move |ws| handle_socket(ws, ua_str, addr))
}

async fn handle_socket(ws: WebSocket, ua: String, addr: SocketAddr) {
 let (mut ws_tx, mut ws_rx) = ws.split();
 let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
 let mut rx = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
 let triggers: Arc<Mutex<HashMap<i64, stream_cancel::Trigger>>> = Default::default();

 let connection_info = super::ConnectionInfo {
  remote_addr: Some(addr),
  user_agent: Some(ua),
  connection_local: Default::default(),
 };

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
  let triggers_clone = triggers.clone();
  let connection_info_clone = connection_info.clone();
  tokio::task::spawn(async move {
   let data = msg.clone().into_data();
   if !data.is_empty() {
    let target_func: Box<dyn crate::RPC> = match bincode::deserialize(&data) {
     Ok(target_func) => target_func,
     Err(e) => {
      if !matches!(msg, Message::Ping(_)) {
       log::error!("websocket deserialize error: {} {:?}", e, msg);
      }
      return;
     }
    };
    let (trigger, tripwire) = stream_cancel::Tripwire::new();
    let trigger = triggers_clone.lock().unwrap().insert(target_func.txid(), trigger);
    drop(triggers_clone);
    if let Some(trigger) = trigger {
     trigger.cancel();
    } else {
     let sender = Box::new(move |response| tx_clone.send(Message::Binary(response)).unwrap());
     target_func.execute(sender, Some(tripwire), Some(connection_info_clone)).await;
    }
   }
  });
 }

 log::info!("websocket disconnected")
}
