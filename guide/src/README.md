# Introduction

<br />

**Turbocharger** is a custom RPC solution that prioritizes development speed.

<br />

It is intended to be used with:

- Rust native executable servers and clients
- Rust WASM clients
- JavaScript clients via WASM (lower priority)

It communicates over:

- WebSockets
- UDP option for small payloads between two native Rust executables

For native executables:

- It prioritizes Linux
- macOS should generally work for development purposes
- No effort is made to support Windows, but PRs are welcome
