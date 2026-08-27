use crate::Error;
use bytes::Bytes;
use futures_util::FutureExt;
use rust_socketio::asynchronous::Client;
use rust_socketio::asynchronous::ClientBuilder;
use rust_socketio::Payload;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

const JANUS_EVENT: &str = "janus";

/// Upper bound on how long we wait for the Socket.IO `open` event before giving
/// up. Generous enough to ride out transient `error` events during the polling
/// handshake (which we no longer treat as fatal), but bounded so a server that
/// only ever emits `error` (e.g. an auth `ConnectError` frame) can't hang the
/// caller forever.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

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
        let url = normalize_scheme(url);
        tracing::debug!("Connecting to {url}");
        let (tx, rx) = mpsc::unbounded_channel::<Bytes>();

        let (open_tx, open_rx) = oneshot::channel::<()>();
        let open_tx = Arc::new(Mutex::new(Some(open_tx)));

        let (fail_tx, fail_rx) = oneshot::channel::<()>();
        let fail_tx = Arc::new(Mutex::new(Some(fail_tx)));

        let socket = ClientBuilder::new(&url)
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
            .on("error", move |payload, _client| {
                // `rust_socketio` emits `error` events for transient, recoverable
                // transport hiccups too (e.g. `IncompleteResponseFromEngineIo`,
                // logged as "EngineIO Error"), not just fatal failures — its own
                // client keeps polling and may still fire `open` afterwards.
                // Treating every `error` as terminal here aborted the connect mid
                // handshake, racing the `open` event. Log for diagnostics only and
                // let the `open`/timeout race decide success.
                async move {
                    tracing::warn!("Socket.IO error event: {payload:?}");
                }
                .boxed()
            })
            .on("close", move |_payload, _client| {
                let fail_tx = fail_tx.clone();
                async move {
                    // A genuine `close` before `open` is a real connection failure.
                    tracing::warn!("Socket.IO connection closed");
                    if let Ok(mut guard) = fail_tx.lock() {
                        if let Some(tx) = guard.take() {
                            let _ = tx.send(());
                        }
                    }
                }
                .boxed()
            })
            .connect()
            .await?;

        let outcome =
            tokio::time::timeout(CONNECT_TIMEOUT, futures_util::future::select(open_rx, fail_rx))
                .await;
        match outcome {
            Ok(futures_util::future::Either::Left((result, _))) => {
                if result.is_err() {
                    tracing::warn!("Socket.IO `open` channel closed before the event fired");
                    return Err(Error::RequestTimeout);
                }
            }
            Ok(futures_util::future::Either::Right((_, _))) => {
                tracing::error!("Socket.IO connection closed before the `open` event fired");
                return Err(Error::RequestTimeout);
            }
            Err(_) => {
                tracing::error!("Socket.IO `open` event did not fire within {CONNECT_TIMEOUT:?}");
                return Err(Error::RequestTimeout);
            }
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

fn normalize_scheme(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("wss://") {
        format!("https://{rest}")
    } else if let Some(rest) = url.strip_prefix("ws://") {
        format!("http://{rest}")
    } else {
        url.to_string()
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

#[cfg(test)]
mod tests {
    use super::normalize_scheme;

    #[test]
    fn maps_websocket_schemes_to_http() {
        assert_eq!(
            normalize_scheme("wss://example.com/janus"),
            "https://example.com/janus"
        );
        assert_eq!(
            normalize_scheme("ws://example.com/janus"),
            "http://example.com/janus"
        );
    }

    #[test]
    fn leaves_http_schemes_untouched() {
        assert_eq!(
            normalize_scheme("https://example.com/janus"),
            "https://example.com/janus"
        );
        assert_eq!(
            normalize_scheme("http://example.com/janus"),
            "http://example.com/janus"
        );
    }
}
