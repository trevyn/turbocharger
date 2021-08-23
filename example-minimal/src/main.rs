use warp::Filter;

#[tokio::main]
async fn main() {
 let routes = warp::path("turbocharger")
  .and(warp::ws())
  .map(|ws: warp::ws::Ws| ws.on_upgrade(turbocharger::accept_warp_ws));

 eprintln!("Serving on http://127.0.0.1:8080");
 warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
