use turbocharger::prelude::*; // includes wasm_bindgen::prelude::*

#[wasm]
pub async fn wasm_get_greeting() -> String {
 "Hello from WASM.".to_string()
}

#[backend]
pub async fn backend_get_greeting() -> String {
 println!("backend_get_greeting called");
 "Hello from backend.".to_string()
}

#[server_only]
#[tokio::main]
async fn main() {
 println!("Serving on http://127.0.0.1:8080");
 warp::serve(turbocharger::warp_routes()).run(([127, 0, 0, 1], 8080)).await;
}
