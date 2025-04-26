#[derive(thiserror::Error, Debug)]
pub enum Error {
    /* Transformed Errors */
    #[cfg(not(target_family = "wasm"))]
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[cfg(not(target_family = "wasm"))]
    #[error("InvalidHeaderValue: {0}")]
    InvalidHeaderValue(#[from] tokio_tungstenite::tungstenite::http::header::InvalidHeaderValue),

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

pub enum JanusError {
    /// Unauthorized (can only happen when using apisecret/auth token)
    Unauthorized,
    /// Unauthorized access to a plugin (can only happen when using auth token)
    UnauthorizedPlugin,
    /// Transport related error
    TransportSpecific,
    /// The request is missing in the message
    MissingRequest,
    /// The Janus core does not support this request
    UnknownRequest,
    /// The payload is not a valid JSON message
    InvalidJson,
    /// The object is not a valid JSON object as expected
    InvalidJsonObject,
    /// A mandatory element is missing in the message
    MissingMandatoryElement,
    /// The request cannot be handled for this webserver path
    InvalidRequestPath,
    /// The session the request refers to doesn't exist
    SessionNotFound,
    /// The handle the request refers to doesn't exist
    HandleNotFound,
    /// The plugin the request wants to talk to doesn't exist
    PluginNotFound,
    /// An error occurring when trying to attach to a plugin and create a handle
    PluginAttach,
    /// An error occurring when trying to send a message/request to the plugin
    PluginMessage,
    /// An error occurring when trying to detach from a plugin and destroy the related handle
    PluginDetach,
    /// The Janus core doesn't support this SDP type
    JsepUnkownType,
    /// The Session Description provided by the peer is invalid
    JsepInvalidSdp,
    /// The stream a trickle candidate for does not exist or is invalid
    TrickleInvalidStream,
    /// A JSON element is of the wrong type (e.g., an integer instead of a string)
    InvalidElementType,
    /// The ID provided to create a new session is already in use
    SessionConflict,
    /// We got an ANSWER to an OFFER we never made
    UnexpectedAnswer,
    /// The auth token the request refers to doesn't exist
    TokenNotFound,
    /// The current request cannot be handled because of not compatible WebRTC state
    WebrtcState,
    /// The server is currently configured not to accept new sessions
    NotAcceptingSessions,
    /// Unknown/undocumented error
    Unkown,
    /// Other error codes, typically plugin specific error codes
    Other(u16),
}

impl From<u16> for JanusError {
    fn from(value: u16) -> Self {
        type __ = JanusError;
        match value {
            403 => __::Unauthorized,
            405 => __::UnauthorizedPlugin,
            450 => __::TransportSpecific,
            452 => __::MissingRequest,
            453 => __::UnknownRequest,
            454 => __::InvalidJson,
            455 => __::InvalidJsonObject,
            456 => __::MissingMandatoryElement,
            457 => __::InvalidRequestPath,
            458 => __::SessionNotFound,
            459 => __::HandleNotFound,
            460 => __::PluginNotFound,
            461 => __::PluginAttach,
            462 => __::PluginMessage,
            463 => __::PluginDetach,
            464 => __::JsepUnkownType,
            465 => __::JsepInvalidSdp,
            466 => __::TrickleInvalidStream,
            467 => __::InvalidElementType,
            468 => __::SessionConflict,
            469 => __::UnexpectedAnswer,
            470 => __::TokenNotFound,
            471 => __::WebrtcState,
            472 => __::NotAcceptingSessions,
            490 => __::Unkown,
            x @ _ => __::Other(x),
        }
    }
}
