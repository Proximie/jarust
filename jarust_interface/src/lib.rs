//! # Jarust Interface
//!
//! Jarust interface contains:
//!
//! - Transport abstraction, you can use the built-in WebSocket interface, restful interface, or bring your own.
//! - Transaction generation abstraction, you can use the built-in transaction generator or bring your own.
//! - DTOs for the Janus API.
//! - Errors
//!

pub mod custom_interface;
pub mod error;
pub mod handle_msg;
pub mod janus_interface;
pub mod japrotocol;
pub mod restful;
pub mod tgenerator;
pub mod transport_trait;
pub mod websocket;

pub(crate) mod transport;

pub type Error = error::Error;
