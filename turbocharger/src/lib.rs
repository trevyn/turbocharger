#[cfg(not(target_arch = "wasm32"))]
pub fn warp_routes() -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
 use warp::Filter;
 warp::path("turbocharger_socket")
  .and(warp::ws())
  .map(|ws: warp::ws::Ws| ws.on_upgrade(accept_connection))
  .boxed()
}

#[cfg(not(target_arch = "wasm32"))]
async fn accept_connection(ws: warp::ws::WebSocket) {}
