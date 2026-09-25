#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct JaConfig {
    /// Url to janus server
    pub url: String,
    /// Janus api secret if any
    pub apisecret: Option<String>,
    /// root path for janus, when using HTTP it should be `janus` unless it was changed
    /// in janus config
    pub server_root: String,
    /// Ring buffer capacity, used when picking WebSocket janus api
    pub capacity: usize,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum JanusAPI {
    WebSocket,
    Restful,
}
