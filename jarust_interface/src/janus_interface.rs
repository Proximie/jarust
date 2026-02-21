use crate::handle_msg::HandleMessage;
use crate::handle_msg::HandleMessageWithJsep;
use crate::japrotocol::JaResponse;
use crate::japrotocol::JaSuccessProtocol;
use crate::japrotocol::PluginInnerData;
use crate::japrotocol::ResponseType;
use crate::japrotocol::ServerInfoRsp;
use crate::tgenerator::GenerateTransaction;
use crate::Error;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

#[cfg(not(target_family = "wasm"))]
pub trait MaybeSend: Send {}
#[cfg(not(target_family = "wasm"))]
impl<T: Send> MaybeSend for T {}

#[cfg(target_family = "wasm")]
pub trait MaybeSend {}
#[cfg(target_family = "wasm")]
impl<T> MaybeSend for T {}

#[cfg(not(target_family = "wasm"))]
pub trait MaybeSync: Sync {}
#[cfg(not(target_family = "wasm"))]
impl<T: Sync> MaybeSync for T {}

#[cfg(target_family = "wasm")]
pub trait MaybeSync {}
#[cfg(target_family = "wasm")]
impl<T> MaybeSync for T {}

pub struct ConnectionParams {
    /// The url of the janus server.
    pub url: String,
    /// The capacity of the connection (for the websocket interface).
    pub capacity: usize,
    /// The api secret (if any).
    pub apisecret: Option<String>,
    /// The server root, it should match the server root of the janus server when choosing the restful interface.
    pub server_root: String,
}

/// [`JanusInterface`] is the main trait that defines the interface for the janus server.
///
/// It acts as a contract to implement different interfaces supported by janus server,
/// full docs: <https://janus.conf.meetecho.com/docs/rest.html>
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
#[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
pub trait JanusInterface: Debug + MaybeSend + MaybeSync + 'static {
    /// Constructs a new interface with the given connection parameters and transaction generator.
    async fn make_interface(
        conn_params: ConnectionParams,
        transaction_generator: impl GenerateTransaction,
    ) -> Result<Self, Error>
    where
        Self: Sized;

    /// Creates a new session with the janus server.
    async fn create(&self, timeout: Duration) -> Result<u64, Error>;

    /// Gets the server info.
    async fn server_info(&self, timeout: Duration) -> Result<ServerInfoRsp, Error>;

    /// Attaches a plugin to the session.
    async fn attach(
        &self,
        session_id: u64,
        plugin_id: String,
        timeout: Duration,
    ) -> Result<(u64, mpsc::UnboundedReceiver<JaResponse>), Error>;

    /// Indicates if the interface has keep alive messages.
    fn has_keep_alive(&self) -> bool;

    /// Send keep alive messages (to keep the connection and the client-server session alive).
    async fn keep_alive(&self, session_id: u64, timeout: Duration) -> Result<(), Error>;

    /// Destroys the session.
    async fn destroy(&self, session_id: u64, timeout: Duration) -> Result<(), Error>;

    /// Sends a one-shot message
    async fn fire_and_forget_msg(&self, message: HandleMessage) -> Result<String, Error>;

    /// Sends a message and waits for acknowledgment.
    async fn send_msg_waiton_ack(
        &self,
        message: HandleMessage,
        timeout: Duration,
    ) -> Result<String, Error>;

    /// Internal method to send a message and wait for the response. Ideally, this shouldn't be internal,
    /// but we can't have a generic return type for this method as it would be considered object-unsafe.
    ///
    /// Being object-unsafe means we can't use it as a trait object (dyn JanusInterface). Therefore, we have to
    /// make this internal, and the public method that uses this one will have a generic return type.
    /// See [`JanusInterfaceImpl::send_msg_waiton_rsp`] for the public method.
    ///
    /// Check this stack overflow answer for the technicalities:
    /// [Why are trait methods with generic type parameters are object unsafe](https://stackoverflow.com/questions/67767207/why-are-trait-methods-with-generic-type-parameters-object-unsafe)
    async fn internal_send_msg_waiton_rsp(
        &self,
        message: HandleMessage,
        timeout: Duration,
    ) -> Result<JaResponse, Error>;

    /// Sends a one-shot message with jsep.
    async fn fire_and_forget_msg_with_jsep(
        &self,
        message: HandleMessageWithJsep,
    ) -> Result<String, Error>;

    /// Sends a message and waits for acknowledgment with jsep.
    async fn send_msg_waiton_ack_with_jsep(
        &self,
        message: HandleMessageWithJsep,
        timeout: Duration,
    ) -> Result<String, Error>;

    /// Sends a top-level handle request.
    ///
    /// This is used for requests like `hangup`, `detach`, etc.
    async fn send_handle_request(&self, request: HandleMessage) -> Result<(), Error>;

    /// Sends a top-level handle request and wait on ack
    ///
    /// This is used for requests like 'trickle' so far.
    async fn send_handle_request_waiton_ack(
        &self,
        request: HandleMessage,
        timeout: Duration,
    ) -> Result<String, Error>;

    /// Returns the name of the interface (for the debug trait)
    fn name(&self) -> Box<str> {
        "Janus Interface".to_string().into_boxed_str()
    }
}

impl dyn JanusInterface {
    /// Sends a message and waits for the response.
    #[tracing::instrument(level = tracing::Level::TRACE, skip_all)]
    pub async fn send_msg_waiton_rsp<R>(
        &self,
        message: HandleMessage,
        timeout: Duration,
    ) -> Result<R, Error>
    where
        R: DeserializeOwned,
    {
        let response = self.internal_send_msg_waiton_rsp(message, timeout).await?;
        let result = match response.janus {
            ResponseType::Success(JaSuccessProtocol::Plugin { plugin_data }) => {
                match plugin_data.data {
                    PluginInnerData::Error { error_code, error } => {
                        tracing::error!(
                            "Plugin response error: {{ code: {error_code}, error: {error} }}"
                        );
                        return Err(Error::PluginResponseError { error_code, error });
                    }
                    PluginInnerData::Data(data) => match serde_json::from_value::<R>(data) {
                        Ok(result) => result,
                        Err(error) => {
                            tracing::error!("Failed to parse with error {error:#?}");
                            return Err(Error::UnexpectedResponse);
                        }
                    },
                }
            }
            _ => {
                tracing::error!("Request failed");
                return Err(Error::UnexpectedResponse);
            }
        };
        Ok(result)
    }
}

#[derive(Clone)]
pub struct JanusInterfaceImpl {
    inner: Arc<dyn JanusInterface>,
}

impl Deref for JanusInterfaceImpl {
    type Target = Arc<dyn JanusInterface>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JanusInterfaceImpl {
    pub fn new(interface: impl JanusInterface) -> Self {
        Self {
            inner: Arc::new(interface),
        }
    }
}

impl Debug for JanusInterfaceImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Interface")
            .field(&self.inner.name())
            .finish()
    }
}
