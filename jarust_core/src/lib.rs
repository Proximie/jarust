//! # Jarust
//!
//! Jarust is a Rust adapter for [Janus WebRTC server](https://github.com/meetecho/janus-gateway).
//!
//! It provides a high-level API to interact with the Janus server.
//!
//! You can use it to connect with the Janus server, create a session,
//! attach a plugin, send messages to the plugin, and handle the incoming messages.
//!
//! ## Customizability
//!
//! Janus supports multiple transports, each transport has a different API to interact with.
//!
//! Jarust was built in a modular manner to support the variations Janus provides. It also has its customizations like the transaction generation strategy.
//!
//! ## Runtime
//!
//! We currently only support the Tokio runtime and are planning to support more runtimes in the future. For that, we've abstracted the runtime-specific code in the [`jarust_rt`] crate.
//!
//! ## Plugins
//!
//! We have a separate crate for Janus plugins, [`jarust_plugins`](https://crates.io/crates/jarust_plugins).
//!

pub mod jaconfig;
pub mod jaconnection;
pub mod jahandle;
mod jakeepalive;
pub mod japlugin;
pub mod jasession;
pub mod prelude;

pub use jarust_interface::tgenerator::GenerateTransaction;

use jaconfig::JaConfig;
use jaconfig::JanusAPI;
use jaconnection::JaConnection;
use jarust_interface::janus_interface::ConnectionParams;
use jarust_interface::janus_interface::JanusInterface;
use jarust_interface::restful::RestfulInterface;
use jarust_interface::websocket::WebSocketInterface;
use tracing::Level;

/// Creates a new connection with janus server from the provided configs.
///
/// ## Example:
///
/// ```rust
/// let config = JaConfig::builder()
///     .url("ws://localhost:8188/ws")
///     .capacity(32)
///     .build();
/// let mut connection = jarust_core::connect(config, ApiInterface::WebSocket, RandomTransactionGenerator).await.unwrap();
/// ```
#[cfg(not(target_family = "wasm"))]
pub async fn connect(
    jaconfig: JaConfig,
    api_interface: JanusAPI,
    transaction_generator: impl GenerateTransaction,
) -> Result<JaConnection, jarust_interface::Error> {
    let conn_params = ConnectionParams {
        url: jaconfig.url,
        capacity: jaconfig.capacity,
        apisecret: jaconfig.apisecret,
        server_root: jaconfig.server_root,
    };
    match api_interface {
        JanusAPI::WebSocket => {
            custom_connect(
                WebSocketInterface::make_interface(conn_params, transaction_generator).await?,
            )
            .await
        }
        JanusAPI::Restful => {
            custom_connect(
                RestfulInterface::make_interface(conn_params, transaction_generator).await?,
            )
            .await
        }
    }
}

/// Creates a new connection with janus server from the provided configs
#[cfg(target_family = "wasm")]
pub async fn connect(
    jaconfig: JaConfig,
    api_interface: JanusAPI,
    transaction_generator: impl GenerateTransaction,
) -> Result<JaConnection, jarust_interface::Error> {
    let conn_params = ConnectionParams {
        url: jaconfig.url,
        capacity: jaconfig.capacity,
        apisecret: jaconfig.apisecret,
        server_root: jaconfig.server_root,
    };
    match api_interface {
        JanusAPI::WebSocket => {
            custom_connect(
                WebSocketInterface::make_interface(conn_params, transaction_generator).await?,
            )
            .await
        }
        JanusAPI::Restful => {
            custom_connect(
                RestfulInterface::make_interface(conn_params, transaction_generator).await?,
            )
            .await
        }
    }
}

/// Creates a new customized connection with janus servers.
#[tracing::instrument(level = Level::TRACE, skip_all)]
pub async fn custom_connect(
    interface: impl JanusInterface,
) -> Result<JaConnection, jarust_interface::Error> {
    JaConnection::open(interface).await
}
