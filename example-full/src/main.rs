use turbocharger::{backend, server_only};
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use turbosql::{select, Turbosql};

#[backend]
#[cfg_attr(not(target_arch = "wasm32"), derive(Turbosql))]
pub struct Person {
 pub rowid: Option<i64>,
 pub name: Option<String>,
}

#[backend]
async fn insert_person(p: Person) -> Result<i64, turbosql::Error> {
 p.insert() // returns rowid
}

#[backend]
async fn get_person(rowid: i64) -> Result<Person, turbosql::Error> {
 select!(Person "WHERE rowid = ?", rowid)
}

#[server_only]
#[tokio::main]
async fn main() {
 #[derive(rust_embed::RustEmbed)]
 #[folder = "build"]
 struct Frontend;

 eprintln!("Serving on http://127.0.0.1:8080");
 opener::open("http://127.0.0.1:8080").ok();
 warp::serve(turbocharger::warp_routes(Frontend)).run(([127, 0, 0, 1], 8080)).await;
}
