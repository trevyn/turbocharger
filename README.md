# Turbocharger

## > WORK IN PROGRESS <

A seamless RPC layer for connecting a web frontend to a Rust backend server via WASM.

`#[turbocharger::turbocharge]` automatically makes *backend* async functions, e.g.:

```rust
#[turbocharger::turbocharge]
async fn get_person(rowid: i64) -> Person {
 // ... any async backend code here;       ...
 // ... query a remote database, API, etc. ...
 turbosql::select!(Person "WHERE rowid = ?", rowid).unwrap()
}
```

available to your JS/TS *frontend* as an async function:

```js
let person = await wasm.get_person(1);
```

and also available internally to a Rust WASM module running on the frontend:

```rust
let person = get_person(1).await;
```

The underlying `wasm-bindgen` implementation provides automatic TypeScript definitions as well, so the pipeline is fully typed all the way through!

## How It Works

Turbocharger uses a WebSocket connection between the WASM module and your backend, serializes requests and responses with `bincode`, and handles multiplexing and dispatch.

Parameters and return values of any types that implement `Serialize`/`Deserialize` and are compatible with `wasm-bindgen` should work, which includes `struct`s but not yet `enum` variants with values / TypeScript discriminated unions, see [wasm-bindgen#2407](https://github.com/rustwasm/wasm-bindgen/issues/2407).

## Usage

Your backend function should be included in both your backend target and your `wasm-pack`/`wasm32-unknown-unknown` target. The `#[turbocharge]` macro effectively generates two implementations of this function; one for the backend that contains the function body you provide as well as the backend WebSocket glue, and one for the frontend WASM module that only provides the frontend WebSocket glue.

## To Do / Future Directions

- Better WebSocket status management / reconnect
- Error handling with `Result::Err` triggering a Promise rejection
- Streaming responses with `futures::stream`
- `Vec<T>` types, see [wasm-bindgen#111](https://github.com/rustwasm/wasm-bindgen/issues/111)
- Anything [`tarpc`](https://github.com/google/tarpc) does, particularly around timeouts, cancellation, etc.