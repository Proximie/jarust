use crate::japrotocol::Jsep;
use serde_json::json;
use serde_json::Value;

pub struct HandleMessage {
    pub session_id: u64,
    pub handle_id: u64,
    pub body: Value,
}

impl HandleMessage {
    /// Full `message` envelope including `session_id`/`handle_id`, used by transports
    /// that carry these in the payload (WebSocket, Socket.IO).
    pub(crate) fn to_message_envelope(&self) -> Value {
        json!({
            "janus": "message",
            "session_id": self.session_id,
            "handle_id": self.handle_id,
            "body": self.body,
        })
    }

    /// Body-only `message` envelope, used by the RESTful transport where
    /// `session_id`/`handle_id` are carried in the URL.
    pub(crate) fn to_message_body(&self) -> Value {
        json!({
            "janus": "message",
            "body": self.body,
        })
    }
}

pub struct HandleMessageWithJsep {
    pub session_id: u64,
    pub handle_id: u64,
    pub body: Value,
    pub jsep: Jsep,
}

impl HandleMessageWithJsep {
    /// Full `message` envelope including `session_id`/`handle_id` and `jsep`, used by
    /// transports that carry these in the payload (WebSocket, Socket.IO).
    pub(crate) fn to_message_envelope(&self) -> Value {
        json!({
            "janus": "message",
            "session_id": self.session_id,
            "handle_id": self.handle_id,
            "body": self.body,
            "jsep": self.jsep,
        })
    }

    /// Body-only `message` envelope including `jsep`, used by the RESTful transport
    /// where `session_id`/`handle_id` are carried in the URL.
    pub(crate) fn to_message_body(&self) -> Value {
        json!({
            "janus": "message",
            "body": self.body,
            "jsep": self.jsep,
        })
    }
}
