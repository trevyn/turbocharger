#[allow(unused_imports)]
use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
use turbocharger::{backend, server_only, wasm_only};
#[cfg(not(target_arch = "wasm32"))]
use turbosql::{select, Turbosql};

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Turbosql))]
pub struct Person {
 pub rowid: Option<i64>,
 pub name: Option<String>,
}

#[wasm_only]
pub fn new_person() -> Person {
 Person::default()
}

#[backend]
pub async fn insert_person(p: Person) -> i32 {
 dbg!(p.insert().unwrap()) as i32 // returns rowid
}

#[backend]
async fn get_person(rowid: i64) -> Person {
 turbosql::select!(Person "WHERE rowid = ?", rowid).unwrap()
}

#[server_only]
#[tokio::main]
async fn main() {
 eprintln!("Serving on http://127.0.0.1:8080");
 warp::serve(turbocharger::warp_routes()).run(([127, 0, 0, 1], 8080)).await;
}
