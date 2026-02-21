#[derive(thiserror::Error, Debug)]
pub enum Error {
    /* Transformed Errors */
    #[cfg(not(target_family = "wasm"))]
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[cfg(not(target_family = "wasm"))]
    #[error("InvalidHeaderValue: {0}")]
    InvalidHeaderValue(#[from] tokio_tungstenite::tungstenite::http::header::InvalidHeaderValue),

    #[error("WebSocket WASM error: {0}")]
    WebSocketWasm(#[from] tokio_tungstenite_wasm::Error),

    #[error("Failed to parse json: {0}")]
    JsonParsingFailure(#[from] serde_json::Error),
    #[error("IO: {0}")]
    IO(#[from] std::io::Error),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /* Custom Errors */
    #[error("Error while parsing an incomplete packet")]
    IncompletePacket,
    #[error("Transport is not opened")]
    TransportNotOpened,
    #[error("Invalid Janus request {{ reason: {reason} }}")]
    InvalidJanusRequest { reason: String },
    #[error("Can't send data in closed channel")]
    SendError,
    #[error("Received an unexpected response")]
    UnexpectedResponse,
    #[error("Janus error {{ code: {code}, reason: {reason}}}")]
    JanusError { code: u16, reason: String },
    #[error("Plugin response error {{ error_code: {error_code}, error: {error} }}")]
    PluginResponseError { error_code: u16, error: String },
    #[error("Request timeout")]
    RequestTimeout,
}
