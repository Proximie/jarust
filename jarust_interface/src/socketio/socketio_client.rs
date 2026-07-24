use crate::Error;
use bytes::Bytes;
use futures_util::FutureExt;
use rust_socketio::asynchronous::Client;
use rust_socketio::asynchronous::ClientBuilder;
use rust_socketio::Payload;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

const JANUS_EVENT: &str = "janus";

pub struct SocketIoClient {
    socket: Option<Client>,
}

impl std::fmt::Debug for SocketIoClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SocketIoClient")
            .field("connected", &self.socket.is_some())
            .finish()
    }
}

impl Default for SocketIoClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SocketIoClient {
    pub fn new() -> Self {
        Self { socket: None }
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    pub async fn connect(&mut self, url: &str) -> Result<mpsc::UnboundedReceiver<Bytes>, Error> {
        tracing::debug!("Connecting to {url}");
        let (tx, rx) = mpsc::unbounded_channel::<Bytes>();

        let (open_tx, open_rx) = oneshot::channel::<()>();
        let open_tx = Arc::new(Mutex::new(Some(open_tx)));

        let socket = ClientBuilder::new(url)
            .on(JANUS_EVENT, move |payload, _client| {
                let tx = tx.clone();
                async move {
                    tracing::debug!("Received on '{JANUS_EVENT}': {payload:?}");
                    forward_payload(payload, &tx);
                }
                .boxed()
            })
            .on("open", move |_payload, _client| {
                let open_tx = open_tx.clone();
                async move {
                    if let Ok(mut guard) = open_tx.lock() {
                        if let Some(tx) = guard.take() {
                            let _ = tx.send(());
                        }
                    }
                }
                .boxed()
            })
            .on("error", |payload, _client| {
                async move {
                    tracing::error!("Socket.IO error event: {payload:?}");
                }
                .boxed()
            })
            .connect()
            .await?;

        if open_rx.await.is_err() {
            tracing::warn!("Socket.IO connection closed before the `open` event fired");
        }

        self.socket = Some(socket);
        Ok(rx)
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    pub async fn send(&self, data: &[u8], _: &str) -> Result<(), Error> {
        let Some(socket) = &self.socket else {
            tracing::error!("Transport not opened!");
            return Err(Error::TransportNotOpened);
        };
        let value: serde_json::Value = serde_json::from_slice(data)?;
        socket.emit(JANUS_EVENT, value).await?;
        Ok(())
    }
}

fn forward_payload(payload: Payload, tx: &mpsc::UnboundedSender<Bytes>) {
    match payload {
        Payload::Text(values) => {
            for value in values {
                forward_json(value, tx);
            }
        }
        Payload::Binary(bytes) => {
            let _ = tx.send(bytes);
        }
        #[allow(deprecated)]
        Payload::String(text) => {
            let _ = tx.send(Bytes::from(text));
        }
    }
}

fn forward_json(value: serde_json::Value, tx: &mpsc::UnboundedSender<Bytes>) {
    match value {
        serde_json::Value::Object(ref map) if map.contains_key("janus") => {
            let _ = tx.send(Bytes::from(value.to_string()));
        }
        serde_json::Value::Array(items) => {
            for item in items {
                if item.get("janus").is_some() {
                    let _ = tx.send(Bytes::from(item.to_string()));
                }
            }
        }
        other => {
            tracing::trace!("Ignoring non-Janus payload element: {other}");
        }
    }
}
