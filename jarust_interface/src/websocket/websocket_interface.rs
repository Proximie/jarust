use super::demuxer::Demuxer;
use super::napmap::NapMap;
use super::router::Router;
use super::tmanager::TransactionManager;
use super::websocket_client::WebSocketClient;
use crate::handle_msg::HandleMessage;
use crate::handle_msg::HandleMessageWithJsep;
use crate::janus_interface::ConnectionParams;
use crate::janus_interface::JanusInterface;
use crate::japrotocol::JaResponse;
use crate::japrotocol::JaSuccessProtocol;
use crate::japrotocol::ResponseType;
use crate::japrotocol::ServerInfoRsp;
use crate::tgenerator::GenerateTransaction;
use crate::tgenerator::TransactionGenerator;
use crate::Error;
use jarust_rt::JaTask;
use serde_json::json;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

#[derive(Debug)]
struct Shared {
    tasks: Vec<JaTask>,
    server_root: String,
    apisecret: Option<String>,
    transaction_generator: TransactionGenerator,
    ack_map: Arc<NapMap<String, JaResponse>>,
    rsp_map: Arc<NapMap<String, JaResponse>>,
}

#[derive(Debug)]
struct Exclusive {
    router: Router,
    ws: WebSocketClient,
    transaction_manager: TransactionManager,
}

#[derive(Debug)]
struct InnerWebSocketInterface {
    shared: Shared,
    exclusive: Mutex<Exclusive>,
}

#[derive(Debug, Clone)]
pub struct WebSocketInterface {
    inner: Arc<InnerWebSocketInterface>,
}

impl WebSocketInterface {
    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    pub async fn send(&self, message: Value) -> Result<String, Error> {
        let (message, transaction) = self.decorate_request(message);

        let path =
            Router::path_from_request(&message).unwrap_or(self.inner.shared.server_root.clone());

        let mut guard = self.inner.exclusive.lock().await;
        guard.transaction_manager.insert(&transaction, &path).await;
        guard.ws.send(message.to_string().as_bytes(), &path).await?;
        tracing::trace!("Sending {message:#?}");
        Ok(transaction)
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip(self, timeout))]
    async fn poll_response(
        &self,
        transaction: &str,
        timeout: Duration,
    ) -> Result<JaResponse, Error> {
        tracing::trace!("Polling response");
        match tokio::time::timeout(
            timeout,
            self.inner.shared.rsp_map.get(transaction.to_string()),
        )
        .await
        {
            Ok(Some(response)) => match response.janus {
                ResponseType::Error { error } => Err(Error::JanusError {
                    error: error.code.into(),
                    reason: error.reason,
                }),
                _ => Ok(response),
            },
            Ok(None) => {
                tracing::error!("Incomplete packet");
                Err(Error::IncompletePacket)
            }
            Err(_) => {
                tracing::error!("Request timeout");
                Err(Error::RequestTimeout)
            }
        }
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip(self, timeout))]
    async fn poll_ack(&self, transaction: &str, timeout: Duration) -> Result<JaResponse, Error> {
        tracing::trace!("Polling ack");
        match tokio::time::timeout(
            timeout,
            self.inner.shared.ack_map.get(transaction.to_string()),
        )
        .await
        {
            Ok(Some(response)) => match response.janus {
                ResponseType::Error { error } => Err(Error::JanusError {
                    error: error.code.into(),
                    reason: error.reason,
                }),
                _ => Ok(response),
            },
            Ok(None) => {
                tracing::error!("Incomplete packet");
                Err(Error::IncompletePacket)
            }
            Err(_) => {
                tracing::error!("Request timeout");
                Err(Error::RequestTimeout)
            }
        }
    }

    fn decorate_request(&self, mut request: Value) -> (Value, String) {
        let transaction = self
            .inner
            .shared
            .transaction_generator
            .generate_transaction();
        if let Some(apisecret) = self.inner.shared.apisecret.clone() {
            request["apisecret"] = apisecret.into();
        };
        request["transaction"] = transaction.clone().into();
        (request, transaction)
    }
}

#[async_trait::async_trait]
impl JanusInterface for WebSocketInterface {
    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn make_interface(
        conn_params: ConnectionParams,
        transaction_generator: impl GenerateTransaction,
    ) -> Result<Self, Error> {
        tracing::debug!("Creating WebSocket Interface");
        let router = Router::new(&conn_params.server_root);
        let mut websocket = WebSocketClient::new();
        let receiver = websocket.connect(&conn_params.url).await?;
        let transaction_manager = TransactionManager::new(conn_params.capacity);
        let transaction_generator = TransactionGenerator::new(transaction_generator);

        let ack_map = Arc::new(NapMap::<String, JaResponse>::new(conn_params.capacity));
        let rsp_map = Arc::new(NapMap::<String, JaResponse>::new(conn_params.capacity));

        let (rsp_sender, mut rsp_receiver) = mpsc::unbounded_channel::<JaResponse>();
        let (ack_sender, mut ack_receiver) = mpsc::unbounded_channel::<JaResponse>();

        let rsp_task = jarust_rt::spawn("Responses gathering task", {
            let rsp_map = rsp_map.clone();
            async move {
                while let Some(rsp) = rsp_receiver.recv().await {
                    if let Some(transaction) = rsp.transaction.clone() {
                        rsp_map.insert(transaction, rsp).await;
                    }
                }
            }
        });

        let ack_task = jarust_rt::spawn("ACKs gathering task", {
            let ack_map = ack_map.clone();
            async move {
                while let Some(rsp) = ack_receiver.recv().await {
                    if let Some(transaction) = rsp.transaction.clone() {
                        ack_map.insert(transaction, rsp).await;
                    }
                }
            }
        });

        let demux_task = jarust_rt::spawn("Demultiplexing task", {
            let router = router.clone();
            let transaction_manager = transaction_manager.clone();
            let demuxer = Demuxer {
                inbound_stream: receiver,
                router,
                rsp_sender,
                ack_sender,
                transaction_manager,
            };
            async move { demuxer.start().await }
        });

        let shared = Shared {
            tasks: vec![demux_task, rsp_task, ack_task],
            server_root: conn_params.server_root,
            apisecret: conn_params.apisecret,
            transaction_generator,
            ack_map,
            rsp_map,
        };
        let exclusive = Exclusive {
            router,
            ws: websocket,
            transaction_manager,
        };
        let inner = InnerWebSocketInterface {
            shared,
            exclusive: Mutex::new(exclusive),
        };
        let this = Self {
            inner: Arc::new(inner),
        };
        Ok(this)
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn create(&self, timeout: Duration) -> Result<u64, Error> {
        let request = json!({
            "janus": "create"
        });

        let transaction = self.send(request).await?;
        let response = self.poll_response(&transaction, timeout).await?;
        let session_id = match response.janus {
            ResponseType::Success(JaSuccessProtocol::Data { data }) => data.id,
            ResponseType::Error { error } => {
                let what = Error::JanusError {
                    error: error.code.into(),
                    reason: error.reason,
                };
                tracing::error!("{what}");
                return Err(what);
            }
            _ => {
                tracing::error!("Unexpected response");
                return Err(Error::UnexpectedResponse);
            }
        };
        Ok(session_id)
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn server_info(&self, timeout: Duration) -> Result<ServerInfoRsp, Error> {
        let request = json!({
            "janus": "info"
        });
        let transaction = self.send(request).await?;
        let response = self.poll_response(&transaction, timeout).await?;
        match response.janus {
            ResponseType::ServerInfo(info) => Ok(*info),
            ResponseType::Error { error } => Err(Error::JanusError {
                error: error.code.into(),
                reason: error.reason,
            }),
            _ => Err(Error::IncompletePacket),
        }
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn attach(
        &self,
        session_id: u64,
        plugin_id: String,
        timeout: Duration,
    ) -> Result<(u64, mpsc::UnboundedReceiver<JaResponse>), Error> {
        let request = json!({
            "janus": "attach",
            "session_id": session_id,
            "plugin": plugin_id
        });
        let transaction = self.send(request).await?;
        let response = self.poll_response(&transaction, timeout).await?;
        let handle_id = match response.janus {
            ResponseType::Success(JaSuccessProtocol::Data { data }) => data.id,
            ResponseType::Error { error } => {
                let what = Error::JanusError {
                    error: error.code.into(),
                    reason: error.reason,
                };
                tracing::error!("{what}");
                return Err(what);
            }
            _ => {
                tracing::error!("Unexpected response");
                return Err(Error::UnexpectedResponse);
            }
        };
        let receiver = self
            .inner
            .exclusive
            .lock()
            .await
            .router
            .add_subroute(&format!("{session_id}/{handle_id}"))
            .await;
        Ok((handle_id, receiver))
    }

    fn has_keep_alive(&self) -> bool {
        true
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn keep_alive(&self, session_id: u64, timeout: Duration) -> Result<(), Error> {
        let request = json!({
            "janus": "keepalive",
            "session_id": session_id
        });
        let transaction = self.send(request).await?;
        self.poll_ack(&transaction, timeout).await?;
        Ok(())
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn destroy(&self, session_id: u64, timeout: Duration) -> Result<(), Error> {
        let request = json!({
            "janus": "destroy",
            "session_id": session_id
        });
        let transaction = self.send(request).await?;
        self.poll_response(&transaction, timeout).await?;
        Ok(())
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn fire_and_forget_msg(&self, message: HandleMessage) -> Result<String, Error> {
        let request = json!({
            "janus": "message",
            "session_id": message.session_id,
            "handle_id": message.handle_id,
            "body": message.body
        });
        let transaction = self.send(request).await?;
        Ok(transaction)
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn send_msg_waiton_ack(
        &self,
        message: HandleMessage,
        timeout: Duration,
    ) -> Result<String, Error> {
        let request = json!({
            "janus": "message",
            "session_id": message.session_id,
            "handle_id": message.handle_id,
            "body": message.body
        });
        let transaction = self.send(request).await?;
        self.poll_ack(&transaction, timeout).await?;
        Ok(transaction)
    }

    async fn internal_send_msg_waiton_rsp(
        &self,
        message: HandleMessage,
        timeout: Duration,
    ) -> Result<JaResponse, Error> {
        let request = json!({
            "janus": "message",
            "session_id": message.session_id,
            "handle_id": message.handle_id,
            "body": message.body
        });
        let transaction = self.send(request).await?;
        self.poll_response(&transaction, timeout).await
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn fire_and_forget_msg_with_jsep(
        &self,
        message: HandleMessageWithJsep,
    ) -> Result<String, Error> {
        let request = json!({
            "janus": "message",
            "session_id": message.session_id,
            "handle_id": message.handle_id,
            "body": message.body,
            "jsep": message.jsep
        });
        let transaction = self.send(request).await?;
        Ok(transaction)
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn send_msg_waiton_ack_with_jsep(
        &self,
        message: HandleMessageWithJsep,
        timeout: Duration,
    ) -> Result<String, Error> {
        let request = json!({
            "janus": "message",
            "session_id": message.session_id,
            "handle_id": message.handle_id,
            "body": message.body,
            "jsep": message.jsep,
        });
        let transaction = self.send(request).await?;
        self.poll_ack(&transaction, timeout).await?;
        Ok(transaction)
    }

    async fn send_handle_request(&self, request: HandleMessage) -> Result<(), Error> {
        let mut req = request.body;
        merge_json(
            &mut req,
            &json!({
                "session_id": request.session_id,
                "handle_id": request.handle_id,
            }),
        );
        _ = self.send(req).await?;
        Ok(())
    }

    async fn send_handle_request_waiton_ack(
        &self,
        request: HandleMessage,
        timeout: Duration,
    ) -> Result<String, Error> {
        let mut req = request.body;
        merge_json(
            &mut req,
            &json!({
                "session_id": request.session_id,
                "handle_id": request.handle_id,
            }),
        );
        let transaction = self.send(req).await?;
        self.poll_ack(&transaction, timeout).await?;
        Ok(transaction)
    }

    fn name(&self) -> Box<str> {
        "WebSocket Interface".to_string().into_boxed_str()
    }
}

impl Drop for InnerWebSocketInterface {
    fn drop(&mut self) {
        self.shared.tasks.iter().for_each(|task| {
            task.cancel();
        });
    }
}

fn merge_json(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), Value::Object(b)) => {
            for (k, v) in b {
                merge_json(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}
