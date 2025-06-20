use crate::JanusId;
use jarust_core::prelude::JaResponse;
use jarust_interface::japrotocol::GenericEvent;
use jarust_interface::japrotocol::JaHandleEvent;
use jarust_interface::japrotocol::PluginInnerData;
use jarust_interface::japrotocol::ResponseType;
use serde::Deserialize;
use serde_json::from_value;
use serde_json::Value;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
#[serde(tag = "streaming")]
enum StreamingEventDto {
    #[serde(rename = "destroyed")]
    DestroyMountpoint { id: JanusId },

    #[serde(rename = "created")]
    CreateMountpoint {
        id: JanusId,
        /// <live|on demand>
        #[serde(rename = "type")]
        mountpoint_type: String,
    },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PluginEvent {
    StreamingEvent(StreamingEvent),
    GenericEvent(GenericEvent),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum StreamingEvent {
    MountpointDestroyed {
        id: JanusId,
    },
    MountpointCreated {
        id: JanusId,
        mountpoint_type: String,
    },
    Error {
        error_code: u16,
        error: String,
    },
    Other(Value),
}

impl TryFrom<JaResponse> for PluginEvent {
    type Error = jarust_interface::Error;

    fn try_from(value: JaResponse) -> Result<Self, Self::Error> {
        match value.janus {
            ResponseType::Event(JaHandleEvent::PluginEvent { plugin_data }) => {
                let streaming_event = match plugin_data.data {
                    PluginInnerData::Error { error_code, error } => {
                        StreamingEvent::Error { error_code, error }
                    }
                    PluginInnerData::Data(data) => {
                        match from_value::<StreamingEventDto>(data.clone()) {
                            Ok(StreamingEventDto::CreateMountpoint {
                                id,
                                mountpoint_type,
                            }) => StreamingEvent::MountpointCreated {
                                id,
                                mountpoint_type,
                            },
                            Ok(StreamingEventDto::DestroyMountpoint { id }) => {
                                StreamingEvent::MountpointDestroyed { id }
                            }
                            Err(_) => StreamingEvent::Other(data),
                        }
                    }
                };
                Ok(PluginEvent::StreamingEvent(streaming_event))
            }
            ResponseType::Event(JaHandleEvent::GenericEvent(event)) => {
                Ok(PluginEvent::GenericEvent(event))
            }
            _ => Err(Self::Error::IncompletePacket),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PluginEvent;

    use crate::streaming::events::StreamingEvent;
    use crate::JanusId;
    use jarust_interface::japrotocol::JaHandleEvent;
    use jarust_interface::japrotocol::JaResponse;
    use jarust_interface::japrotocol::PluginData;
    use jarust_interface::japrotocol::PluginInnerData;
    use jarust_interface::japrotocol::ResponseType;
    use serde_json::json;

    #[test]
    fn it_parse_mountpoint_created() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.streaming".to_string(),
                    data: PluginInnerData::Data(json!({
                        "streaming": "created",
                        "id": 63807u64,
                        "type": "live",
                    })),
                },
            }),
            jsep: None,
            transaction: None,
            session_id: None,
            sender: None,
        };
        let event: PluginEvent = rsp.try_into().unwrap();
        assert_eq!(
            event,
            PluginEvent::StreamingEvent(StreamingEvent::MountpointCreated {
                id: JanusId::Uint(63807u64.try_into().unwrap()),
                mountpoint_type: "live".to_string(),
            })
        );
    }

    #[test]
    fn it_parse_mountpoint_destroyed() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.streaming".to_string(),
                    data: PluginInnerData::Data(json!({
                        "streaming": "destroyed",
                        "id": 63807u64,
                    })),
                },
            }),
            jsep: None,
            transaction: None,
            session_id: None,
            sender: None,
        };
        let event: PluginEvent = rsp.try_into().unwrap();
        assert_eq!(
            event,
            PluginEvent::StreamingEvent(StreamingEvent::MountpointDestroyed {
                id: JanusId::Uint(63807u64.try_into().unwrap()),
            })
        );
    }

    #[test]
    fn it_parse_error() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.streaming".to_string(),
                    data: PluginInnerData::Error {
                        error_code: 456,
                        error: "Can't add 'rtp' stream, error creating data source stream"
                            .to_string(),
                    },
                },
            }),
            jsep: None,
            transaction: None,
            session_id: None,
            sender: None,
        };
        let event: PluginEvent = rsp.try_into().unwrap();
        assert_eq!(
            event,
            PluginEvent::StreamingEvent(StreamingEvent::Error {
                error_code: 456,
                error: "Can't add 'rtp' stream, error creating data source stream".to_string()
            })
        );
    }

    #[test]
    fn it_parse_unsupported_event_as_other() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.streaming".to_string(),
                    data: PluginInnerData::Data(json!({
                        "streaming": "jarust_rocks",
                        "jarust": "rocks"
                    })),
                },
            }),
            jsep: None,
            transaction: None,
            session_id: None,
            sender: None,
        };
        let event: PluginEvent = rsp.try_into().unwrap();
        assert_eq!(
            event,
            PluginEvent::StreamingEvent(StreamingEvent::Other(json!({
                "streaming": "jarust_rocks",
                "jarust": "rocks"
            })))
        );
    }
}
