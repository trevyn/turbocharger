use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn get_wasm_greeting() -> String {
 "Hello from WASM.".to_string()
}

#[wasm_bindgen]
pub fn get_wasm_greeting_sync() -> String {
 "Hello from WASM.".to_string()
}
