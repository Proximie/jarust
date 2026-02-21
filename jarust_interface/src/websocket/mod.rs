pub(crate) mod demuxer;
pub(crate) mod napmap;
pub(crate) mod ringbuf_map;
pub(crate) mod router;
pub(crate) mod tmanager;

#[cfg(not(target_family = "wasm"))]
pub mod native;

#[cfg(target_family = "wasm")]
pub mod wasm;

#[cfg(not(target_family = "wasm"))]
pub use native::WebSocketInterface;

#[cfg(target_family = "wasm")]
pub use wasm::WebSocketInterface;
