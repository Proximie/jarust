# jarust WASM Demo

A browser demo that mirrors the `jarust/examples/secure_ws.rs` example, compiled to WebAssembly via `wasm-pack`.

It connects to the public Janus demo server (`wss://janus.conf.meetecho.com/ws`), creates a session, attaches to the **Echo Test** plugin, fires a couple of messages, and logs the events to the browser console (and the on-page log box).

## Build

Install `wasm-pack` if you haven't:

```sh
cargo install wasm-pack
```

Build the WASM package (outputs to `pkg/`):

```sh
cd jarust_wasm_demo
wasm-pack build --target web --no-typescript
```

## Serve

Any static file server works. For example with Python:

```sh
python3 -m http.server 8080
```

Then open `http://localhost:8080` in your browser and click **Connect & Run**.

Logs from `tracing` are forwarded through `tracing-wasm` to the browser console and displayed in the on-page log box.

## How it maps to `secure_ws.rs`

| `secure_ws.rs`                              | `jarust_wasm_demo/src/lib.rs`                    |
|---------------------------------------------|--------------------------------------------------|
| `#[tokio::main]`                            | `#[wasm_bindgen(start)]` async fn               |
| `tracing_subscriber::fmt()`                 | `tracing_wasm::set_as_global_default()`         |
| `JanusAPI::WebSocket`                       | `JanusAPI::WebSocket`                            |
| `tokio::spawn` loop with `time::interval`   | Single-shot messages (no interval on WASM)      |
| `event_receiver.recv()` loop                | Drains 3 events then exits                      |
