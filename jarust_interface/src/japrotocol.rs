use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

/// The top-level response
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct JaResponse {
    #[serde(flatten)]
    pub janus: ResponseType,
    pub transaction: Option<String>,
    pub session_id: Option<u64>,
    pub sender: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsep: Option<Jsep>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "janus")]
pub enum ResponseType {
    #[serde(rename = "error")]
    Error { error: ErrorResponse },
    // Wrapped in a box so the entire enum isn't as large as this variant
    // e.g: Ack that doesn't contain any data will have allocate the same space as ServerInfo
    #[serde(rename = "server_info")]
    ServerInfo(Box<ServerInfoRsp>),
    #[serde(rename = "ack")]
    Ack,
    #[serde(rename = "success")]
    Success(JaSuccessProtocol),
    #[serde(untagged)]
    Event(JaHandleEvent),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub reason: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "janus")]
pub enum JaSuccessProtocol {
    #[serde(untagged)]
    Data { data: JaData },
    #[serde(untagged)]
    Plugin {
        #[serde(rename = "plugindata")]
        plugin_data: PluginData,
    },
    #[serde(untagged)]
    Empty {},
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JaData {
    pub id: u64,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PluginData {
    pub plugin: String,
    pub data: PluginInnerData,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginInnerData {
    Error { error_code: u16, error: String },
    // Gotcha
    // The pattern matching is done in the order of the variants,
    // this field wraps a generic value, so it will always match.
    Data(Value),
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(tag = "janus")]
pub enum JaHandleEvent {
    #[serde(rename = "event")]
    PluginEvent {
        #[serde(rename = "plugindata")]
        plugin_data: PluginData,
    },
    #[serde(untagged)]
    GenericEvent(GenericEvent),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "janus")]
pub enum GenericEvent {
    #[serde(rename = "detached")]
    Detached,
    /// The PeerConnection was closed, either by Janus or by the user/application, and as such cannot be used anymore.
    #[serde(rename = "hangup")]
    Hangup,
    /// Whether Janus is receiving (receiving: true/false) audio/video (type: "audio/video") on this PeerConnection.
    #[serde(rename = "media")]
    Media,
    #[serde(rename = "timeout")]
    Timeout,
    /// ICE and DTLS succeeded, and so Janus correctly established a PeerConnection with the user/application.
    #[serde(rename = "webrtcup")]
    WebrtcUp,
    /// Whether Janus is reporting trouble sending/receiving (uplink: true/false) media on this PeerConnection.
    #[serde(rename = "slowlink")]
    Slowlink {
        media: String,
        uplink: bool,
        lost: u32,
    },
    #[serde(rename = "trickle")]
    Trickle,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum JsepType {
    #[serde(rename = "offer")]
    Offer,
    #[serde(rename = "answer")]
    Answer,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Jsep {
    #[serde(rename = "type")]
    pub jsep_type: JsepType,
    pub sdp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trickle: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize)]
pub struct Candidate {
    #[serde(rename = "sdpMid")]
    pub sdp_mid: String,
    #[serde(rename = "sdpMLineIndex")]
    pub sdp_mline_index: u32,
    pub candidate: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ServerInfoRsp {
    pub name: String,
    pub version: u64,
    #[serde(rename = "version_string")]
    pub version_string: String,
    pub author: String,
    pub commit_hash: String,
    pub compile_time: String,
    pub log_to_stdout: bool,
    pub log_to_file: bool,
    #[serde(rename = "data_channels")]
    pub data_channels: bool,
    pub accepting_new_sessions: bool,
    pub session_timeout: u64,
    pub reclaim_session_timeout: u64,
    pub candidates_timeout: u64,
    pub server_name: String,
    pub local_ip: String,
    pub ipv6: bool,
    pub ice_lite: bool,
    pub ice_tcp: bool,
    pub ice_nomination: String, /* could be enum when we know the variants */
    pub ice_keepalive_conncheck: bool,
    pub full_trickle: bool,
    pub mdns_enabled: bool,
    pub min_nack_queue: u64,
    pub twcc_period: u64,
    pub dtls_mtu: u64,
    pub static_event_loops: u64,
    #[serde(rename = "api_secret")]
    pub api_secret: bool,
    #[serde(rename = "auth_token")]
    pub auth_token: bool,
    #[serde(rename = "event_handlers")]
    pub event_handlers: bool,
    #[serde(rename = "opaqueid_in_api")]
    pub opaqueid_in_api: bool,
    pub dependencies: HashMap<String, String>,
    pub transports: HashMap<String, MetaData>,
    pub plugins: HashMap<String, MetaData>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct MetaData {
    pub name: String,
    pub author: String,
    pub description: String,
    pub version_string: String,
    pub version: u64,
}

#[cfg(test)]
mod tests {
    use super::GenericEvent;
    use super::JaData;
    use super::JaHandleEvent;
    use super::JaResponse;
    use super::JaSuccessProtocol;
    use super::Jsep;
    use super::JsepType;
    use super::PluginData;
    use super::ResponseType;
    use crate::japrotocol::PluginInnerData;
    use serde_json::json;

    #[test]
    fn it_parse_create_connection_rsp() {
        let rsp = json!({
            "janus": "success",
            "transaction": "7be89359-8c3f-44fc-93a6-72e35bb56058",
            "data": {
                "id": 5486640424129986u64
            }
        });
        let actual_rsp = serde_json::from_value::<JaResponse>(rsp).unwrap();
        let expected = JaResponse {
            janus: ResponseType::Success(JaSuccessProtocol::Data {
                data: JaData {
                    id: 5486640424129986u64,
                },
            }),
            transaction: Some("7be89359-8c3f-44fc-93a6-72e35bb56058".to_string()),
            sender: None,
            session_id: None,
            jsep: None,
        };
        assert_eq!(actual_rsp, expected);
    }

    #[test]
    fn it_parse_attach_rsp() {
        let rsp = json!({
            "janus": "success",
            "session_id": 1706796313061627u64,
            "transaction": "151f9362-3d12-45e5-ba02-b91a38be5a06",
            "data": {
                "id": 7548423276295183u64
            }
        });
        let actual_rsp = serde_json::from_value::<JaResponse>(rsp).unwrap();
        let expected = JaResponse {
            janus: ResponseType::Success(JaSuccessProtocol::Data {
                data: JaData {
                    id: 7548423276295183u64,
                },
            }),
            transaction: Some("151f9362-3d12-45e5-ba02-b91a38be5a06".to_string()),
            sender: None,
            session_id: Some(1706796313061627u64),
            jsep: None,
        };
        assert_eq!(actual_rsp, expected);
    }

    #[test]
    fn it_parse_echotest_event() {
        let event = json!({
            "janus": "event",
            "sender": 3010144072065778u64,
            "transaction": "c7bb120f-ed4e-4e00-b8de-bfc3e66f098e",
            "session_id": 8643988533991908u64,
            "plugindata": {
                "plugin": "janus.plugin.echotest",
                "data": {
                    "echotest": "event",
                    "result": "ok"
                }
            },
            "jsep": {
                "type": "answer",
                "sdp": "random_sdp"
            }
        });
        let actual_event = serde_json::from_value::<JaResponse>(event).unwrap();
        let expected = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.echotest".to_string(),
                    data: PluginInnerData::Data(json!({
                        "echotest": "event",
                        "result": "ok"
                    })),
                },
            }),
            transaction: Some("c7bb120f-ed4e-4e00-b8de-bfc3e66f098e".to_string()),
            sender: Some(3010144072065778u64),
            session_id: Some(8643988533991908u64),
            jsep: Some(Jsep {
                sdp: "random_sdp".to_string(),
                trickle: None,
                jsep_type: JsepType::Answer,
            }),
        };
        assert_eq!(actual_event, expected);
    }

    #[test]
    fn it_parse_detached_event() {
        let event = json!({
            "janus": "detached",
            "sender": 5373520011480655u64,
            "session_id": 3889473834879521u64
        });
        let actual_event = serde_json::from_value::<JaResponse>(event).unwrap();
        let expected = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::GenericEvent(GenericEvent::Detached)),
            transaction: None,
            sender: Some(5373520011480655u64),
            session_id: Some(3889473834879521u64),
            jsep: None,
        };
        assert_eq!(actual_event, expected);
    }

    #[test]
    fn it_parse_webrtcup_event() {
        let event = json!({
            "janus": "webrtcup",
            "sender": 2676358135723942u64,
            "session_id": 1942958911060866u64
        });
        let actual_event = serde_json::from_value::<JaResponse>(event).unwrap();
        let expected = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::GenericEvent(GenericEvent::WebrtcUp)),
            transaction: None,
            sender: Some(2676358135723942u64),
            session_id: Some(1942958911060866u64),
            jsep: None,
        };
        assert_eq!(actual_event, expected);
    }

    #[test]
    fn it_parse_error_event() {
        let event = json!({
           "janus": "success",
           "session_id": 2158724686674557u64,
           "transaction": "nNbmsbj33zLY",
           "sender": 77797716144085u64,
           "plugindata": {
              "plugin": "janus.plugin.streaming",
              "data": {
                 "error_code": 456,
                 "error": "Can't add 'rtp' stream, error creating data source stream"
              }
           }
        });
        let actual_event = serde_json::from_value::<JaResponse>(event).unwrap();
        let expected = JaResponse {
            janus: ResponseType::Success(JaSuccessProtocol::Plugin {
                plugin_data: PluginData {
                    plugin: "janus.plugin.streaming".to_string(),
                    data: PluginInnerData::Error {
                        error_code: 456,
                        error: "Can't add 'rtp' stream, error creating data source stream"
                            .to_string(),
                    },
                },
            }),
            transaction: Some("nNbmsbj33zLY".to_string()),
            sender: Some(77797716144085u64),
            session_id: Some(2158724686674557u64),
            jsep: None,
        };
        assert_eq!(actual_event, expected);
    }

    #[test]
    fn it_parse_slow_link_event() {
        let event = json!({
            "janus": "slowlink",
            "sender": 2676358135723942u64,
            "session_id": 1942958911060866u64,
            "uplink": true,
            "media": "audio",
            "lost": 10
        });
        let actual_event = serde_json::from_value::<JaResponse>(event).unwrap();
        let expected = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::GenericEvent(GenericEvent::Slowlink {
                media: "audio".to_string(),
                uplink: true,
                lost: 10,
            })),
            transaction: None,
            sender: Some(2676358135723942u64),
            session_id: Some(1942958911060866u64),
            jsep: None,
        };
        assert_eq!(actual_event, expected);
    }
}
