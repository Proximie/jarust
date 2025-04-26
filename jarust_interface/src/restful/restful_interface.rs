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
    apisecret: Option<String>,
    transaction_generator: TransactionGenerator,
    client: reqwest::Client,
    url: String,
}

#[derive(Debug)]
struct Exclusive {
    tasks: Vec<JaTask>,
}

#[derive(Debug)]
struct InnerResultfulInterface {
    shared: Shared,
    exclusive: Mutex<Exclusive>,
}

#[derive(Debug, Clone)]
pub struct RestfulInterface {
    inner: Arc<InnerResultfulInterface>,
}

impl RestfulInterface {
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
impl JanusInterface for RestfulInterface {
    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn make_interface(
        conn_params: ConnectionParams,
        transaction_generator: impl GenerateTransaction,
    ) -> Result<Self, Error> {
        tracing::debug!("Creating new Restful Interface");
        let client = reqwest::Client::new();
        let transaction_generator = TransactionGenerator::new(transaction_generator);
        let shared = Shared {
            apisecret: conn_params.apisecret,
            transaction_generator,
            client,
            url: format!("{}/{}", conn_params.url, conn_params.server_root),
        };
        let exclusive = Exclusive { tasks: Vec::new() };
        let inner = InnerResultfulInterface {
            shared,
            exclusive: Mutex::new(exclusive),
        };
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn create(&self, timeout: Duration) -> Result<u64, Error> {
        let url = &self.inner.shared.url;
        let request = json!({"janus": "create"});
        let (request, _) = self.decorate_request(request);

        let response = self
            .inner
            .shared
            .client
            .post(url.to_string())
            .json(&request)
            .timeout(timeout)
            .send()
            .await?
            .json::<JaResponse>()
            .await?;

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
        let url = &self.inner.shared.url;
        let response = self
            .inner
            .shared
            .client
            .get(format!("{url}/info"))
            .timeout(timeout)
            .send()
            .await?
            .json::<JaResponse>()
            .await?;
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
        let url = &self.inner.shared.url;
        let request = json!({
            "janus": "attach",
            "plugin": plugin_id
        });
        let (request, _) = self.decorate_request(request);

        let response = self
            .inner
            .shared
            .client
            .post(format!("{url}/{session_id}"))
            .json(&request)
            .timeout(timeout)
            .send()
            .await?
            .json::<JaResponse>()
            .await?;
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
        let (tx, rx) = mpsc::unbounded_channel();

        let handle = jarust_rt::spawn("Long polling", {
            let client = self.inner.shared.client.clone();
            let url = url.clone();

            async move {
                loop {
                    if let Ok(response) = client
                        .get(format!("{url}/{session_id}?maxev=5"))
                        .send()
                        .await
                    {
                        if let Ok(res) = response.json::<Vec<JaResponse>>().await {
                            for r in res {
                                let _ = tx.send(r);
                            }
                        }
                    };
                }
            }
        });

        self.inner.exclusive.lock().await.tasks.push(handle);

        Ok((handle_id, rx))
    }

    fn has_keep_alive(&self) -> bool {
        false
    }

    async fn keep_alive(&self, _: u64, _: Duration) -> Result<(), Error> {
        Ok(())
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn destroy(&self, session_id: u64, timeout: Duration) -> Result<(), Error> {
        let url = &self.inner.shared.url;
        let request = json!({
            "janus": "destroy"
        });
        let (request, _) = self.decorate_request(request);

        self.inner
            .shared
            .client
            .post(format!("{url}/{session_id}"))
            .json(&request)
            .timeout(timeout)
            .send()
            .await?;
        Ok(())
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn fire_and_forget_msg(&self, message: HandleMessage) -> Result<String, Error> {
        let url = &self.inner.shared.url;
        let session_id = message.session_id;
        let handle_id = message.handle_id;

        let request = json!({
            "janus": "message",
            "body": message.body
        });
        let (request, transaction) = self.decorate_request(request);
        self.inner
            .shared
            .client
            .post(format!("{url}/{session_id}/{handle_id}"))
            .json(&request)
            .send()
            .await?;
        Ok(transaction)
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn send_msg_waiton_ack(
        &self,
        message: HandleMessage,
        timeout: Duration,
    ) -> Result<String, Error> {
        let url = &self.inner.shared.url;
        let session_id = message.session_id;
        let handle_id = message.handle_id;

        let request = json!({
            "janus": "message",
            "body": message.body
        });
        let (request, transaction) = self.decorate_request(request);
        self.inner
            .shared
            .client
            .post(format!("{url}/{session_id}/{handle_id}"))
            .json(&request)
            .timeout(timeout)
            .send()
            .await?
            .json::<JaResponse>()
            .await?;
        Ok(transaction)
    }

    async fn internal_send_msg_waiton_rsp(
        &self,
        message: HandleMessage,
        timeout: Duration,
    ) -> Result<JaResponse, Error> {
        let url = &self.inner.shared.url;
        let session_id = message.session_id;
        let handle_id = message.handle_id;

        let request = json!({
            "janus": "message",
            "body": message.body
        });
        let (request, _) = self.decorate_request(request);
        let response = self
            .inner
            .shared
            .client
            .post(format!("{url}/{session_id}/{handle_id}"))
            .json(&request)
            .timeout(timeout)
            .send()
            .await?
            .json::<JaResponse>()
            .await?;
        Ok(response)
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn fire_and_forget_msg_with_jsep(
        &self,
        message: HandleMessageWithJsep,
    ) -> Result<String, Error> {
        let url = &self.inner.shared.url;
        let session_id = message.session_id;
        let handle_id = message.handle_id;

        let request = json!({
            "janus": "message",
            "body": message.body,
            "jsep": message.jsep
        });
        let (request, transaction) = self.decorate_request(request);
        self.inner
            .shared
            .client
            .post(format!("{url}/{session_id}/{handle_id}"))
            .json(&request)
            .send()
            .await?;
        Ok(transaction)
    }

    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    async fn send_msg_waiton_ack_with_jsep(
        &self,
        message: HandleMessageWithJsep,
        timeout: Duration,
    ) -> Result<String, Error> {
        let url = &self.inner.shared.url;
        let session_id = message.session_id;
        let handle_id = message.handle_id;

        let request = json!({
            "janus": "message",
            "body": message.body,
            "jsep": message.jsep
        });
        let (request, transaction) = self.decorate_request(request);
        self.inner
            .shared
            .client
            .post(format!("{url}/{session_id}/{handle_id}"))
            .json(&request)
            .timeout(timeout)
            .send()
            .await?
            .json::<JaResponse>()
            .await?;
        Ok(transaction)
    }

    async fn send_handle_request(&self, request: HandleMessage) -> Result<(), Error> {
        let url = &self.inner.shared.url;
        let session_id = request.session_id;
        let handle_id = request.handle_id;

        let (request, _) = self.decorate_request(request.body);
        _ = self
            .inner
            .shared
            .client
            .post(format!("{url}/{session_id}/{handle_id}"))
            .json(&request)
            .send()
            .await?
            .json::<JaResponse>()
            .await?;
        Ok(())
    }

    async fn send_handle_request_waiton_ack(
        &self,
        request: HandleMessage,
        timeout: Duration,
    ) -> Result<String, Error> {
        let url = &self.inner.shared.url;
        let session_id = request.session_id;
        let handle_id = request.handle_id;

        let (request, transaction) = self.decorate_request(request.body);
        _ = self
            .inner
            .shared
            .client
            .post(format!("{url}/{session_id}/{handle_id}"))
            .json(&request)
            .timeout(timeout)
            .send()
            .await?
            .json::<JaResponse>()
            .await?;
        Ok(transaction)
    }

    fn name(&self) -> Box<str> {
        "Restful Interface".to_string().into_boxed_str()
    }
}

impl Drop for Exclusive {
    fn drop(&mut self) {
        for task in self.tasks.drain(..) {
            task.cancel();
        }
    }
}
