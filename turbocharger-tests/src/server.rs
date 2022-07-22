mod app;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
 let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8888));
 let app = axum::routing::Router::new()
  .route("/turbocharger_socket", axum::routing::get(turbocharger::ws_handler));
 axum::Server::bind(&addr)
  .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
  .await
  .unwrap();
}
