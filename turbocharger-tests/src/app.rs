#![allow(non_snake_case)]
#![cfg_attr(feature = "wasm", allow(dead_code))]

use turbocharger::prelude::*;

#[backend(js)]
pub async fn run_test() -> i32 {
 println!("in run_test");
 42
}
