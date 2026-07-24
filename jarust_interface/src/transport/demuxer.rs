use super::router::Router;
use super::tmanager::TransactionManager;
use crate::japrotocol::JaResponse;
use crate::japrotocol::ResponseType;
use crate::Error;
use bytes::Bytes;
use tokio::sync::mpsc;

pub(crate) struct Demuxer {
    pub(crate) inbound_stream: mpsc::UnboundedReceiver<Bytes>,
    pub(crate) router: Router,
    pub(crate) rsp_sender: mpsc::UnboundedSender<JaResponse>,
    pub(crate) ack_sender: mpsc::UnboundedSender<JaResponse>,
    pub(crate) transaction_manager: TransactionManager,
}

impl Demuxer {
    /// Async task to handle demultiplexing of the inbound stream
    #[tracing::instrument(name = "incoming_message", level = tracing::Level::TRACE, skip_all)]
    pub(crate) async fn start(self) -> Result<(), Error> {
        let mut stream = self.inbound_stream;
        while let Some(next) = stream.recv().await {
            let Ok(incoming_event) = std::str::from_utf8(&next) else {
                tracing::error!("Incomplete packet received");
                continue;
            };

            tracing::trace!("Received {incoming_event}");

            // Parse the incoming message
            match serde_json::from_str::<JaResponse>(incoming_event) {
                Ok(response) => match response.clone().janus {
                    ResponseType::Error { error } => {
                        tracing::error!("{error:#?}");
                        _ = self.rsp_sender.send(response.clone());
                        _ = self.ack_sender.send(response);
                    }
                    ResponseType::Ack => {
                        _ = self.ack_sender.send(response);
                    }
                    ResponseType::Success(_) | ResponseType::ServerInfo(_) => {
                        _ = self.rsp_sender.send(response);
                    }
                    ResponseType::Event(_) => {
                        if let Err(what) =
                            Demuxer::demux_event(response, &self.router, &self.transaction_manager)
                                .await
                        {
                            tracing::error!("Error demuxing message: {what}");
                        }
                    }
                },
                Err(what) => {
                    tracing::error!("Error parsing response: {what}");
                }
            };
        }
        Ok(())
    }

    /// Route the message to the proper channel
    async fn demux_event(
        message: JaResponse,
        router: &Router,
        transaction_manager: &TransactionManager,
    ) -> Result<(), Error> {
        // Check if we have a pending transaction and demux to the proper route
        if let Some(transaction) = message.transaction.clone() {
            if let Some(path) = transaction_manager.get(&transaction).await {
                router.pub_subroute(&path, message).await?;
                return Ok(());
            }
        }

        // Try get the route from the response
        if let Some(path) = Router::path_from_response(message.clone()) {
            router.pub_subroute(&path, message).await?;
            return Ok(());
        }
        Ok(())
    }
}
