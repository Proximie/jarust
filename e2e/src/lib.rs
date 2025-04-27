use jarust::core::jaconfig::JanusAPI;
use tracing_subscriber::EnvFilter;

/// For debugging e2e tests
pub fn init_tracing_subscriber() {
    let env_filter = EnvFilter::from_default_env()
        .add_directive("jarust_core=trace".parse().unwrap())
        .add_directive("jarust_plugins=trace".parse().unwrap())
        .add_directive("jarust_interface=trace".parse().unwrap())
        .add_directive("jarust_rt=trace".parse().unwrap());
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ServerUrl {
    MultistreamWebsocket,
    MultistreamRestful,
    LegacyWebsocket,
    LegacyRestful,
}

impl ServerUrl {
    pub fn url(&self) -> &'static str {
        match self {
            ServerUrl::MultistreamWebsocket => "ws://localhost:8188/ws",
            ServerUrl::MultistreamRestful => "http://localhost:8088",
            ServerUrl::LegacyWebsocket => "ws://localhost:9188/ws",
            ServerUrl::LegacyRestful => "http://localhost:9088",
        }
    }

    pub fn api(&self) -> JanusAPI {
        match self {
            ServerUrl::MultistreamWebsocket | ServerUrl::LegacyWebsocket => JanusAPI::WebSocket,
            ServerUrl::MultistreamRestful | ServerUrl::LegacyRestful => JanusAPI::Restful,
        }
    }

    pub fn is_legacy(&self) -> bool {
        matches!(self, ServerUrl::LegacyWebsocket | ServerUrl::LegacyRestful)
    }

    pub fn is_multistream(&self) -> bool {
        matches!(
            self,
            ServerUrl::MultistreamWebsocket | ServerUrl::MultistreamRestful
        )
    }
}
