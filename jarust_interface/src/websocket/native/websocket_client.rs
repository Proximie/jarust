use super::connector;
use crate::Error;
use bytes::Bytes;
use futures_util::stream::SplitSink;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use jarust_rt::JaTask;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

#[derive(Debug)]
pub struct WebSocketClient {
    sender: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
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
        let mut request = url.into_client_request()?;
        let headers = request.headers_mut();
        headers.insert("Sec-Websocket-Protocol", "janus-protocol".parse()?);
        let stream = connector::connect_async(request).await?;

        let (sender, mut receiver) = stream.split();
        let (tx, rx) = mpsc::unbounded_channel();

        let task = jarust_rt::spawn("WebSocket incoming messages", async move {
            while let Some(Ok(message)) = receiver.next().await {
                if let Message::Text(text) = message {
                    let _ = tx.send(text.into());
                }
            }
        });

        self.sender = Some(sender);
        self.task = Some(task);
        Ok(rx)
    }

    pub async fn send(&mut self, data: &[u8], _: &str) -> Result<(), Error> {
        let item = Message::Binary(data.to_vec().into());
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
