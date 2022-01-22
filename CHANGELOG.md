# Changelog

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
