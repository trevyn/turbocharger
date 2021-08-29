use turbocharger::{backend, server_only, wasm_only};
#[allow(unused_imports)]
use wasm_bindgen::prelude::*;

#[wasm_only]
pub async fn get_local_greeting1() -> String {
 "Hello from WASM one.".to_string()
}

#[wasm_only]
pub async fn get_local_greeting2() -> String {
 "Hello from WASM two.".to_string()
}

#[backend]
pub async fn get_backend_test() -> String {
 "Hello from get_backend_test.".to_string()
}

#[backend]
pub async fn get_backend_test_no_retval() {}

#[backend]
pub async fn get_backend_test_with_delay() -> String {
 tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
 "Hello from get_backend_test_with_delay; 1 second of delay happened!".to_string()
}

#[backend]
pub async fn get_backend_test_with_string(name: String) -> String {
 format!("Hello from get_backend_test_with_string, {}!", name).to_string()
}

#[backend]
pub async fn get_backend_test_with_i64_i32(one: i64, two: i32) -> String {
 format!("Hello from get_backend_test_with_i64_i32, one:{}, two:{}!", one, two).to_string()
}

#[server_only]
#[tokio::main]
async fn main() {
 eprintln!("Serving on http://127.0.0.1:8080");
 warp::serve(turbocharger::warp_routes()).run(([127, 0, 0, 1], 8080)).await;
}
