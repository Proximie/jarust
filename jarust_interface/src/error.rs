use std::fmt::Display;

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
    #[error("Janus error {{ error: {error}, reason: {reason}}}")]
    JanusError { error: JanusError, reason: String },
    #[error("Plugin response error {{ error_code: {error_code}, error: {error} }}")]
    PluginResponseError { error_code: u16, error: String },
    #[error("Request timeout")]
    RequestTimeout,
}

#[derive(Debug)]
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
        match value {
            403 => JanusError::Unauthorized,
            405 => JanusError::UnauthorizedPlugin,
            450 => JanusError::TransportSpecific,
            452 => JanusError::MissingRequest,
            453 => JanusError::UnknownRequest,
            454 => JanusError::InvalidJson,
            455 => JanusError::InvalidJsonObject,
            456 => JanusError::MissingMandatoryElement,
            457 => JanusError::InvalidRequestPath,
            458 => JanusError::SessionNotFound,
            459 => JanusError::HandleNotFound,
            460 => JanusError::PluginNotFound,
            461 => JanusError::PluginAttach,
            462 => JanusError::PluginMessage,
            463 => JanusError::PluginDetach,
            464 => JanusError::JsepUnkownType,
            465 => JanusError::JsepInvalidSdp,
            466 => JanusError::TrickleInvalidStream,
            467 => JanusError::InvalidElementType,
            468 => JanusError::SessionConflict,
            469 => JanusError::UnexpectedAnswer,
            470 => JanusError::TokenNotFound,
            471 => JanusError::WebrtcState,
            472 => JanusError::NotAcceptingSessions,
            490 => JanusError::Unkown,
            x @ _ => JanusError::Other(x),
        }
    }
}

impl Display for JanusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let description = match self {
            JanusError::Unauthorized => "403 (Unauthorized)",
            JanusError::UnauthorizedPlugin => "405 (Unauthorized access to a plugin)",
            JanusError::TransportSpecific => "450 (Transport related error)",
            JanusError::MissingRequest => "452 (The request is missing in the message)",
            JanusError::UnknownRequest => "453 (The Janus core does not support this request)",
            JanusError::InvalidJson => "454 (The payload is not a valid JSON message)",
            JanusError::InvalidJsonObject => {
                "455 (The object is not a valid JSON object as expected)"
            }
            JanusError::MissingMandatoryElement => {
                "456 (A mandatory element is missing in the message)"
            }
            JanusError::InvalidRequestPath => {
                "457 (The request cannot be handled for this webserver path)"
            }
            JanusError::SessionNotFound => "458 (The session the request refers to doesn't exist)",
            JanusError::HandleNotFound => "459 (The handle the request refers to doesn't exist)",
            JanusError::PluginNotFound => {
                "460 (The plugin the request wants to talk to doesn't exist)"
            }
            JanusError::PluginAttach => "461 (Error attaching to a plugin)",
            JanusError::PluginMessage => "462 (Error sending a message/request to the plugin)",
            JanusError::PluginDetach => "463 (Error detaching from a plugin)",
            JanusError::JsepUnkownType => "464 (The Janus core doesn't support this SDP type)",
            JanusError::JsepInvalidSdp => {
                "465 (The Session Description provided by the peer is invalid)"
            }
            JanusError::TrickleInvalidStream => {
                "466 (The stream a trickle candidate for does not exist or is invalid)"
            }
            JanusError::InvalidElementType => "467 (A JSON element is of the wrong type)",
            JanusError::SessionConflict => {
                "468 (The ID provided to create a new session is already in use)"
            }
            JanusError::UnexpectedAnswer => {
                "469 (Received an ANSWER to an OFFER that was never made)"
            }
            JanusError::TokenNotFound => "470 (The auth token the request refers to doesn't exist)",
            JanusError::WebrtcState => "471 (Incompatible WebRTC state for the current request)",
            JanusError::NotAcceptingSessions => "472 (The server is not accepting new sessions)",
            JanusError::Unkown => "490 (Unknown/undocumented error)",
            JanusError::Other(code) => return write!(f, "Other error with code: {}", code),
        };
        write!(f, "{}", description)
    }
}
