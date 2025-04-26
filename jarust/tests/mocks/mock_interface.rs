use async_trait::async_trait;
use jarust::core::prelude::JaResponse;
use jarust::core::GenerateTransaction;
use jarust::interface::error::Error;
use jarust::interface::handle_msg::HandleMessage;
use jarust::interface::handle_msg::HandleMessageWithJsep;
use jarust::interface::janus_interface::ConnectionParams;
use jarust::interface::janus_interface::JanusInterface;
use jarust::interface::japrotocol::JaSuccessProtocol;
use jarust::interface::japrotocol::ResponseType;
use jarust::interface::japrotocol::ServerInfoRsp;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

#[derive(Debug, Default)]
pub struct Exclusive {
    create_rsp: Option<JaResponse>,
    attach_rsp: Option<JaResponse>,
    server_info_rsp: Option<ServerInfoRsp>,
    handles_rx: HashMap<u64, UnboundedSender<JaResponse>>,
}

#[derive(Debug, Default)]
pub struct InnerMockInterface {
    exclusive: Mutex<Exclusive>,
}

#[derive(Debug, Default, Clone)]
pub struct MockInterface {
    inner: Arc<InnerMockInterface>,
}

#[allow(dead_code)]
impl MockInterface {
    pub async fn mock_create_rsp(&self, rsp: JaResponse) {
        self.inner.exclusive.lock().await.create_rsp = Some(rsp);
    }

    pub async fn mock_attach_rsp(&self, rsp: JaResponse) {
        self.inner.exclusive.lock().await.attach_rsp = Some(rsp);
    }

    pub async fn mocker_server_info_rsp(&self, rsp: ServerInfoRsp) {
        self.inner.exclusive.lock().await.server_info_rsp = Some(rsp);
    }

    pub async fn mock_event(&self, handle_id: u64, rsp: JaResponse) {
        if let Some(tx) = self.inner.exclusive.lock().await.handles_rx.get(&handle_id) {
            tx.send(rsp).unwrap();
        }
    }
}

#[async_trait]
impl JanusInterface for MockInterface {
    async fn make_interface(
        _: ConnectionParams,
        _: impl GenerateTransaction,
    ) -> Result<Self, jarust::interface::Error>
    where
        Self: Sized,
    {
        let exclusive = Mutex::new(Exclusive::default());
        let inner = InnerMockInterface { exclusive };
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    async fn create(&self, _timeout: Duration) -> Result<u64, jarust::interface::Error> {
        let Some(rsp) = self.inner.exclusive.lock().await.create_rsp.clone() else {
            panic!("Create response is not set");
        };
        let session_id = match rsp.janus {
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

    async fn server_info(
        &self,
        _timeout: Duration,
    ) -> Result<ServerInfoRsp, jarust::interface::Error> {
        let Some(rsp) = self.inner.exclusive.lock().await.server_info_rsp.clone() else {
            panic!("Server info response is not set");
        };
        Ok(rsp)
    }

    async fn attach(
        &self,
        _session_id: u64,
        _plugin_id: String,
        _timeout: Duration,
    ) -> Result<(u64, mpsc::UnboundedReceiver<JaResponse>), jarust::interface::Error> {
        let Some(rsp) = self.inner.exclusive.lock().await.attach_rsp.clone() else {
            panic!("Attach response is not set");
        };
        let handle_id = match rsp.janus {
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
        let (tx, rx) = mpsc::unbounded_channel();
        self.inner
            .exclusive
            .lock()
            .await
            .handles_rx
            .insert(handle_id, tx);
        Ok((handle_id, rx))
    }

    fn has_keep_alive(&self) -> bool {
        true
    }

    async fn keep_alive(
        &self,
        _session_id: u64,
        _timeout: Duration,
    ) -> Result<(), jarust::interface::Error> {
        todo!("Keep alive is not implemented");
    }

    async fn destroy(
        &self,
        _session_id: u64,
        _timeout: Duration,
    ) -> Result<(), jarust::interface::Error> {
        todo!("Destroy is not implemented");
    }

    async fn fire_and_forget_msg(
        &self,
        _message: HandleMessage,
    ) -> Result<String, jarust::interface::Error> {
        todo!("Fire and forget is not implemented");
    }

    async fn send_msg_waiton_ack(
        &self,
        _message: HandleMessage,
        _timeout: Duration,
    ) -> Result<String, jarust::interface::Error> {
        todo!("Send message wait on ack is not implemented");
    }

    async fn internal_send_msg_waiton_rsp(
        &self,
        _message: HandleMessage,
        _timeout: Duration,
    ) -> Result<JaResponse, jarust::interface::Error> {
        todo!("Internal send message wait on response is not implemented");
    }

    async fn fire_and_forget_msg_with_jsep(
        &self,
        _message: HandleMessageWithJsep,
    ) -> Result<String, jarust::interface::Error> {
        todo!("Fire and forget with jsep is not implemented");
    }

    async fn send_msg_waiton_ack_with_jsep(
        &self,
        _message: HandleMessageWithJsep,
        _timeout: Duration,
    ) -> Result<String, jarust::interface::Error> {
        todo!("Send message wait on ack with jsep is not implemented");
    }

    async fn send_handle_request(&self, _request: HandleMessage) -> Result<(), Error> {
        todo!("Send handle request is not implemented");
    }

    async fn send_handle_request_waiton_ack(
        &self,
        _request: HandleMessage,
        _timeout: Duration,
    ) -> Result<String, Error> {
        todo!("Send handle request and waiting on ack is not implemented");
    }
}
