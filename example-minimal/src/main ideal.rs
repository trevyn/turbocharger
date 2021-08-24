use turbocharger::{wasm, backend, server_only};

/// Call from JavaScript as `await wasm.get_local_greeting()`
#[wasm]
pub async fn get_local_greeting() -> String {
 "Hello from WASM.".to_string()
}

/// Call from JavaScript as `await backend.get_remote_greeting()`
#[backend]
pub async fn get_remote_greeting() -> String {
 eprintln!("get_remote_greeting called");
 "Hello from backend.".to_string()
}

#[server_only]
#[tokio::main]
async fn main() {
 eprintln!("Serving on http://127.0.0.1:8080");
 warp::serve(turbocharger::warp_routes()).run(([127, 0, 0, 1], 8080)).await;
}
