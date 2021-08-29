# Turbocharger

## > WORK IN PROGRESS <

Autogenerated async RPC bindings that instantly connect a JS frontend to a Rust backend service via WebSockets and WASM.

Makes a Rust _backend_ function, e.g.:

```rust
#[turbocharger::backend]
async fn get_person(id: i64) -> Person {
 // ... write any async backend code here; ...
 // ... query a remote database, API, etc. ...
 Person { name: "Bob", age: 21 }
}
```

instantly available, with _no additional boilerplate_, to a frontend as

- an async JavaScript function
- with full TypeScript type definitions
- that calls the backend over the network:

```js
// export function get_person(id: number): Promise<Person>;

let person = await backend.get_person(1);
```

Works with any types that are [supported by](https://rustwasm.github.io/docs/wasm-bindgen/reference/types.html) `wasm-bindgen`, which includes most basic types and custom `struct`s with fields of supported types, but [not yet](https://github.com/rustwasm/wasm-bindgen/pull/2631) `enum` variants with values (which would come out the other end as TypeScript discriminated unions).

## How It Works

A proc macro auto-generates a frontend `wasm-bindgen` module, which serializes the JS function call parameters with `bincode`. These requests are sent over a shared WebSocket connection to a provided `warp` endpoint on the backend server, which calls your Rust function and serializes the response. This is sent back over the WebSocket and resolves the Promise returned by the original function call.

Multiple async requests can be simultaneously in-flight over a single multiplexed connection; it all just works.

## Complete Example: A full SQLite-powered backend with frontend bindings

This is the complete code that's necessary, but the full project setup and JS build pipeline are in `example-turbosql/`; run with `cargo run --bin example-turbosql`

### `main.rs`

```rust
use turbocharger::prelude::*;

#[backend]
#[derive(turbosql::Turbosql, Default)]
struct Person {
 rowid: Option<i64>,
 name: Option<String>
}

#[backend]
async fn insert_person(p: Person) -> Result<i64, turbosql::Error> {
 p.insert() // returns rowid
}

#[backend]
async fn get_person(rowid: i64) -> Result<Person, turbosql::Error> {
 turbosql::select!(Person "WHERE rowid = ?", rowid)
}

#[server_only]
#[tokio::main]
async fn main() {
 eprintln!("Serving on http://127.0.0.1:8080");
 warp::serve(turbocharger::warp_routes()).run(([127, 0, 0, 1], 8080)).await;
}
```

### `index.js`

```js
import turbocharger_init, * as backend from "./turbocharger_generated";

(async () => {
 await turbocharger_init();
 let person = Object.assign(new backend.Person(), { name: "Bob" });
 let rowid = await backend.insert_person(person);
 console.log((await backend.get_person(rowid)).toJSON());
})();
```

## Usage

Your `main.rs` file is the entry point for both the server `bin` target and a `wasm-bindgen` `lib` target. The `#[backend]` macro outputs three functions:

- Your function, unchanged, for the server `bin` target; you can call it directly from other server code if you wish.
- An internal function for the server `bin` target providing the RPC dispatch glue.
- A `#[wasm_bindgen]` function for the frontend `lib` target that makes the RPC call and delivers the response.

Because the project is compiled to both `wasm32-unknown-unknown` and the host triple, all functions and structs in `main.rs` should be annotated with one of `#[backend]`, `#[server_only]`, or `#[wasm_only]`.

## Error Handling

`#[backend]` functions that need to return an error can return a `Result<T, E>` where `T` is a `wasm-bindgen`-compatible type and `E` is a type that implements `std::error::Error`, including `Box<dyn std::error::Error>>` and `anyhow::Error`. Errors crossing the network boundary are converted to a `String` representation on the server via their `to_string()` method and delivered as a Promise rejection on the JS side.

## Server

Currently, the server side is batteries-included with `warp`, but this could be decoupled in the future. If this decoupling would be useful to you, please open a GitHub issue describing a use case.

## WASM-only functions

You can also easily add standard `#[wasm-bindgen]`-style Rust functions to the WASM module, accessible from the frontend only:

```rust
#[turbocharger::wasm_only]
async fn get_wasm_greeting() -> String {
 "Hello from WASM".to_string()
}
```

## To Do / Future Directions

- Better WebSocket status management / reconnect
- Streaming responses with `futures::stream`
- Many things [`tarpc`](https://github.com/google/tarpc) does, particularly around timeouts and cancellation.

### License: MIT OR Apache-2.0 OR CC0-1.0 (public domain)
