use crate::Error;
use bytes::Bytes;
use futures_util::stream::SplitSink;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use jarust_rt::JaTask;
use tokio::sync::mpsc;
use tokio_tungstenite_wasm::Message;
use tokio_tungstenite_wasm::WebSocketStream;

pub struct WebSocketClient {
    sender: Option<SplitSink<WebSocketStream, Message>>,
    task: Option<JaTask>,
}

impl Default for WebSocketClient {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketClient {
    pub fn new() -> Self {
        Self {
            sender: None,
            task: None,
        }
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    pub async fn connect(&mut self, url: &str) -> Result<mpsc::UnboundedReceiver<Bytes>, Error> {
        tracing::debug!("Connecting to {url}");
        let stream =
            tokio_tungstenite_wasm::connect_with_protocols(url, &["janus-protocol"]).await?;

        let (sender, mut receiver) = stream.split();
        let (tx, rx) = mpsc::unbounded_channel();

        let task = jarust_rt::spawn("WebSocket incoming messages", async move {
            while let Some(Ok(message)) = receiver.next().await {
                if let Message::Text(text) = message {
                    let _ = tx.send(Bytes::from(text));
                }
            }
        });

        self.sender = Some(sender);
        self.task = Some(task);
        Ok(rx)
    }

    pub async fn send(&mut self, data: &[u8], _: &str) -> Result<(), Error> {
        let text = String::from_utf8_lossy(data).into_owned();
        let item = Message::Text(text.into());
        if let Some(sender) = &mut self.sender {
            sender.send(item).await?;
        } else {
            tracing::error!("Transport not opened!");
            return Err(Error::TransportNotOpened);
        }
        Ok(())
    }
}

impl Drop for WebSocketClient {
    #[tracing::instrument(parent = None, level = tracing::Level::TRACE, skip(self))]
    fn drop(&mut self) {
        if let Some(join_handle) = self.task.take() {
            tracing::debug!("Dropping wss transport");
            join_handle.cancel();
        }
    }
}
