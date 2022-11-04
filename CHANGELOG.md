# Changelog

## Unreleased

### Changed

- MSRV is now Rust 1.65

## 0.4.0 - 2022-07-22

### Changed

- `use turbocharger::prelude::*` now imports everything needed.
- `#[backend]` will now by default only provide a Rust WASM frontend function stub. Use `#[backend(js)]` to additionally provide a JS frontend function stub. The JS frontend is more restrictive regarding supported types.
- Use the `remote_addr!()` and `user_agent!()` macros in `#[backend]` functions to get the remote address and user agent.
- Renamed `axum_server` feature to `axum`.
- MSRV is now Rust 1.62

### Added

- Added `connection_local!()` macro to store connection-local data in `#[backend]` functions.

## 0.3.0 - 2022-03-05

### Changed

- Changed server support from `warp` to `axum`.

### Added

- Implicit API description is saved in a `backend_api.rs` file in your project root. You can check this into source control to keep track of API changes.
- Added automatic on-the-fly TLS certificate generation with Let's Encrypt, based on the received TLS SNI.
- Added the ability to stream `Result` types, e.g. `impl Stream<Item = Result<i32, tracked::StringError>>`.
- Allow explicitly setting the backend Websocket URL, e.g. `backend.set_socket_url("ws://localhost:8080/turbocharger_socket");`.
- Backend functions now have access to `remote_addr` and `user_agent` values.

### Improved

- Improved streaming response robustness.

## 0.2.0 - 2022-01-22

### Added

- Added experimental streaming responses.
- Added experimental UDP server.

### Changed

- Simplified `#[server_only]` and `#[wasm_only]` macros to essentially only be shorthand for `#[cfg(not(target_arch = "wasm32"))]` and `#[cfg(target_arch = "wasm32")]` respectively.
- MSRV is now Rust 1.56

### Fixed

- Use value of JS `window.location` to connect to correct socket URL.

## 0.1.0 - 2021-09-10

- Initial release! 🎉
