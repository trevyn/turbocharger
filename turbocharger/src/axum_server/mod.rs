mod tls;

use axum::{
 body::{boxed, Full},
 extract::{
  ws::{Message, WebSocket, WebSocketUpgrade},
  TypedHeader,
 },
 handler::Handler,
 headers,
 http::{header, StatusCode, Uri},
 response::{IntoResponse, Response},
 routing::{get, Router},
 Server,
};
use futures::{SinkExt, StreamExt, TryFutureExt};
use rust_embed::RustEmbed;
use std::marker::PhantomData;
use std::net::SocketAddr;

/// Convenience function to run a full server with static files from `rust_embed` and the Turbocharger WebSocket.
pub async fn serve<A: 'static + RustEmbed>(addr: &SocketAddr) {
 let app = Router::new()
  .route("/turbocharger_socket", get(ws_handler))
  .fallback(rust_embed_handler::<A>.into_service());

 Server::bind(addr).serve(app.into_make_service()).await.unwrap();
}

/// Convenience function to run a full server with static files from `rust_embed` and the Turbocharger WebSocket.
pub async fn serve_tls<A: 'static + RustEmbed>(addr: &SocketAddr) {
 let app = Router::new()
  .route("/turbocharger_socket", get(ws_handler))
  .fallback(rust_embed_handler::<A>.into_service());

 tls::serve(addr, app).await.unwrap();
}

/// Axum handler for serving static files from rust_embed.
pub async fn rust_embed_handler<A: RustEmbed>(uri: Uri) -> impl IntoResponse {
 let mut path = uri.path().trim_start_matches('/').to_string();
 if path.is_empty() {
  path = "index.html".to_string();
 }
 StaticFile::<_, A>(path, PhantomData)
}

struct StaticFile<T, A>(pub T, PhantomData<A>);

impl<T, A> IntoResponse for StaticFile<T, A>
where
 T: Into<String>,
 A: RustEmbed,
{
 fn into_response(self) -> Response {
  let path = self.0.into();
  match A::get(path.as_str()) {
   Some(content) => {
    let body = boxed(Full::from(content.data));
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Response::builder().header(header::CONTENT_TYPE, mime.as_ref()).body(body).unwrap()
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
) -> impl IntoResponse {
 if let Some(TypedHeader(user_agent)) = user_agent {
  log::debug!("connected: {}", user_agent.as_str());
 }

 ws.on_upgrade(handle_socket)
}

async fn handle_socket(ws: WebSocket) {
 let (mut ws_tx, mut ws_rx) = ws.split();
 let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
 let mut rx = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
 let (_trigger, tripwire) = stream_cancel::Tripwire::new();

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
  let tripwire_clone = tripwire.clone();
  tokio::task::spawn(async move {
   let msg = msg.into_data();
   if !msg.is_empty() {
    let target_func: Box<dyn crate::RPC> = bincode::deserialize(&msg).unwrap();
    let sender = Box::new(move |response| tx_clone.send(Message::Binary(response)).unwrap());
    target_func.execute(sender, Some(tripwire_clone)).await;
   }
  });
 }

 log::info!("websocket disconnected")
}
