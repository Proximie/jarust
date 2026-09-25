//! A raw, transport-agnostic byte pipe that the [`CustomInterface`] drives.
//!
//! Janus itself only speaks a handful of native transports (WebSocket, RESTful, raw
//! sockets). Anything else -- a Socket.IO gateway shim, a bespoke relay, a transport
//! living on the other side of an FFI boundary -- can be plugged in by implementing
//! [`Transport`] and handing it to [`CustomInterface`], which layers the full Janus
//! protocol (transactions, demultiplexing, routing, response polling) on top.
//!
//! [`CustomInterface`]: crate::custom_interface::CustomInterface

use crate::janus_interface::MaybeSend;
use crate::janus_interface::MaybeSync;
use crate::Error;
use bytes::Bytes;
use std::fmt::Debug;
use tokio::sync::mpsc;

/// A raw byte transport carrying Janus messages.
///
/// Implementors move bytes; they do not need to understand the Janus protocol. Each
/// item yielded by the receiver returned from [`connect`](Transport::connect) must be
/// the bytes of exactly one `{"janus": ...}` JSON message.
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
#[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
pub trait Transport: Debug + MaybeSend + MaybeSync + 'static {
    /// Establishes the connection to `url` and returns the inbound stream of raw Janus
    /// payloads.
    async fn connect(&self, url: &str) -> Result<mpsc::UnboundedReceiver<Bytes>, Error>;

    /// Sends one request over the transport.
    ///
    /// `path` is advisory routing context (e.g. `"{session_id}/{handle_id}"`) derived
    /// from the request. Transports that don't multiplex on a path -- such as a single
    /// Socket.IO `janus` event -- may ignore it.
    async fn send(&self, data: &[u8], path: &str) -> Result<(), Error>;
}
