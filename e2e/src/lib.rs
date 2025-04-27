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
pub enum TestingEnv {
    Multistream(JanusAPI),
    Legacy(JanusAPI),
}

impl TestingEnv {
    pub fn url(&self) -> &'static str {
        match self {
            Self::Multistream(JanusAPI::WebSocket) => "ws://localhost:8188/ws",
            Self::Multistream(JanusAPI::Restful) => "http://localhost:8088",
            Self::Legacy(JanusAPI::WebSocket) => "ws://localhost:9188/ws",
            Self::Legacy(JanusAPI::Restful) => "http://localhost:9088",
        }
    }

    pub fn api(&self) -> JanusAPI {
        match self {
            Self::Multistream(api) | Self::Legacy(api) => *api,
        }
    }

    pub fn is_legacy(&self) -> bool {
        matches!(self, Self::Legacy(_))
    }

    pub fn is_multistream(&self) -> bool {
        matches!(self, Self::Multistream(_))
    }
}
