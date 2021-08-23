#[cfg(not(target_arch = "wasm32"))]
pub async fn accept_warp_ws(ws: warp::ws::WebSocket) {}
