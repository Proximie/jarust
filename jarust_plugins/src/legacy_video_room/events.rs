use super::responses::LegacyVideoRoomPublisher;
use crate::JanusId;
use jarust_core::prelude::JaResponse;
use jarust_interface::japrotocol::GenericEvent;
use jarust_interface::japrotocol::JaHandleEvent;
use jarust_interface::japrotocol::Jsep;
use jarust_interface::japrotocol::PluginInnerData;
use jarust_interface::japrotocol::ResponseType;
use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
#[serde(tag = "videoroom")]
enum LegacyVideoRoomEventDto {
    #[serde(rename = "joined")]
    Joined {
        id: JanusId,
        room: JanusId,
        private_id: Option<u64>,
        description: Option<String>,
        publishers: Vec<LegacyVideoRoomPublisher>,
    },
    #[serde(rename = "attached")]
    SubscriberAttached {
        id: JanusId,
        room: JanusId,
        display: Option<String>,
    },
    #[serde(rename = "slow_link")]
    SlowLink,
    #[serde(rename = "event")]
    Event(InnerLegacyVideoRoomEvent),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
#[serde(untagged)]
enum InnerLegacyVideoRoomEvent {
    Configured {
        configured: String,
        room: JanusId,
    },
    NewPublishers {
        room: JanusId,
        publishers: Vec<LegacyVideoRoomPublisher>,
    },
    Unpublished {
        room: JanusId,
        unpublished: JanusId,
    },
    Started {
        room: JanusId,
        started: String,
    },
    Leaving {
        room: JanusId,
        leaving: String,
        reason: String,
    },
    Kicked {
        kicked: JanusId,
        room: JanusId,
    },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum LegacyVideoRoomEvent {
    RoomJoined {
        /// unique ID of the new participant
        id: JanusId,
        /// ID of the room the participant joined into
        room: JanusId,
        /// display name of the new participant
        description: Option<String>,
        private_id: Option<u64>,
        publishers: Vec<LegacyVideoRoomPublisher>,
        jsep: Option<Jsep>,
    },
    Configured {
        room: JanusId,
        jsep: Option<Jsep>,
    },
    NewPublishers {
        room: JanusId,
        publishers: Vec<LegacyVideoRoomPublisher>,
    },
    SubscriberAttached {
        id: JanusId,
        room: JanusId,
        display: Option<String>,
        jsep: Jsep,
    },
    SlowLink,
    Unpublished {
        room: JanusId,
        unpublished: JanusId,
    },
    SubscriberStarted {
        room: JanusId,
        started: String,
    },
    Leaving {
        room: JanusId,
        reason: String,
    },
    Kicked {
        room: JanusId,
        participant: JanusId,
    },
    Error {
        error_code: u16,
        error: String,
    },
    Other(Value),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PluginEvent {
    GenericEvent(GenericEvent),
    LegacyVideoRoomEvent(LegacyVideoRoomEvent),
}

impl TryFrom<JaResponse> for PluginEvent {
    type Error = jarust_interface::Error;

    fn try_from(value: JaResponse) -> Result<Self, Self::Error> {
        match value.janus {
            ResponseType::Event(JaHandleEvent::PluginEvent { plugin_data }) => {
                Ok(PluginEvent::LegacyVideoRoomEvent(match plugin_data.data {
                    PluginInnerData::Error { error_code, error } => {
                        LegacyVideoRoomEvent::Error { error_code, error }
                    }
                    PluginInnerData::Data(data) => {
                        match serde_json::from_value::<LegacyVideoRoomEventDto>(data.clone()) {
                            Ok(event) => match event {
                                LegacyVideoRoomEventDto::Joined {
                                    id,
                                    room,
                                    private_id,
                                    description,
                                    publishers,
                                } => LegacyVideoRoomEvent::RoomJoined {
                                    id,
                                    room,
                                    private_id,
                                    description,
                                    publishers,
                                    jsep: value.jsep,
                                },
                                LegacyVideoRoomEventDto::SubscriberAttached {
                                    id,
                                    room,
                                    display,
                                } => {
                                    if let Some(jsep) = value.jsep {
                                        LegacyVideoRoomEvent::SubscriberAttached {
                                            id,
                                            room,
                                            display,
                                            jsep,
                                        }
                                    } else {
                                        LegacyVideoRoomEvent::Other(data)
                                    }
                                }
                                LegacyVideoRoomEventDto::SlowLink => LegacyVideoRoomEvent::SlowLink,
                                LegacyVideoRoomEventDto::Event(event) => match event {
                                    InnerLegacyVideoRoomEvent::Configured { room, .. } => {
                                        LegacyVideoRoomEvent::Configured {
                                            room,
                                            jsep: value.jsep,
                                        }
                                    }
                                    InnerLegacyVideoRoomEvent::NewPublishers {
                                        room,
                                        publishers,
                                    } => LegacyVideoRoomEvent::NewPublishers { room, publishers },
                                    InnerLegacyVideoRoomEvent::Unpublished {
                                        room,
                                        unpublished,
                                    } => LegacyVideoRoomEvent::Unpublished { room, unpublished },
                                    InnerLegacyVideoRoomEvent::Started { room, started } => {
                                        LegacyVideoRoomEvent::SubscriberStarted { room, started }
                                    }
                                    InnerLegacyVideoRoomEvent::Leaving { room, reason, .. } => {
                                        LegacyVideoRoomEvent::Leaving { room, reason }
                                    }
                                    InnerLegacyVideoRoomEvent::Kicked { room, kicked } => {
                                        LegacyVideoRoomEvent::Kicked {
                                            room,
                                            participant: kicked,
                                        }
                                    }
                                },
                            },
                            Err(_) => LegacyVideoRoomEvent::Other(data),
                        }
                    }
                }))
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
    use super::*;
    use jarust_interface::japrotocol::JsepType;
    use serde_json::json;

    #[test]
    fn parse_joined_with_jsep() {
        let raw_event = json!({
            "janus": "event",
            "session_id": 7323526979899781u64,
            "sender": 7967725809069290u64,
            "jsep": {
                "type": "answer",
                "sdp": "test_sdp"
            },
            "plugindata": {
                "plugin": "janus.plugin.videoroom",
                "data": {
                    "videoroom": "joined",
                    "room": 8146468u64,
                    "description": "A brand new description!",
                    "id": 1337,
                    "publishers": [],
                }
            }
        });
        let event: PluginEvent = serde_json::from_value::<JaResponse>(raw_event)
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(
            event,
            PluginEvent::LegacyVideoRoomEvent(LegacyVideoRoomEvent::RoomJoined {
                room: JanusId::Uint(8146468.into()),
                description: Some("A brand new description!".to_string()),
                id: JanusId::Uint(1337.into()),
                private_id: None,
                publishers: vec![],
                jsep: Some(Jsep {
                    jsep_type: JsepType::Answer,
                    sdp: "test_sdp".to_string(),
                    trickle: None
                })
            })
        )
    }

    #[test]
    fn parse_joined_without_jsep() {
        let raw_event = json!({
            "janus": "event",
            "session_id": 7323526979899781u64,
            "sender": 7967725809069290u64,
            "plugindata": {
                "plugin": "janus.plugin.videoroom",
                "data": {
                    "videoroom": "joined",
                    "room": 8146468u64,
                    "description": "A brand new description!",
                    "id": 1337,
                    "private_id": 4113762326u64,
                    "publishers": [],
                }
            }
        });
        let event: PluginEvent = serde_json::from_value::<JaResponse>(raw_event)
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(
            event,
            PluginEvent::LegacyVideoRoomEvent(LegacyVideoRoomEvent::RoomJoined {
                room: JanusId::Uint(8146468.into()),
                description: Some("A brand new description!".to_string()),
                id: JanusId::Uint(1337.into()),
                private_id: Some(4113762326),
                publishers: vec![],
                jsep: None
            })
        )
    }

    #[test]
    fn parse_new_publishers() {
        let raw_event = json!({
            "janus": "event",
            "session_id": 7323526979899781u64,
            "sender": 7967725809069290u64,
            "plugindata": {
                "plugin": "janus.plugin.videoroom",
                "data": {
                    "videoroom": "event",
                    "room": 8146468u64,
                    "private_id": 4113762326u64,
                    "publishers": [
                        {
                            "id": 1337,
                            "display": "A brand new publisher",
                            "substream": 1
                        }
                    ]
                }
            }
        });
        let event: PluginEvent = serde_json::from_value::<JaResponse>(raw_event)
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(
            event,
            PluginEvent::LegacyVideoRoomEvent(LegacyVideoRoomEvent::NewPublishers {
                room: JanusId::Uint(8146468.into()),
                publishers: vec![LegacyVideoRoomPublisher {
                    id: JanusId::Uint(1337.into()),
                    display: Some("A brand new publisher".to_string()),
                    substream: Some(1)
                }]
            })
        )
    }

    #[test]
    fn parse_subscriber_attached() {
        let raw_event = json!({
            "janus": "event",
            "session_id": 7323526979899781u64,
            "sender": 7967725809069290u64,
            "jsep": {
                "type": "offer",
                "sdp": "test_subscriber_sdp"
            },
            "plugindata": {
                "plugin": "janus.plugin.videoroom",
                "data": {
                    "videoroom": "attached",
                    "id": 1337,
                    "room": 8146468u64,
                    "display": "Test Display Name"
                }
            }
        });
        let event: PluginEvent = serde_json::from_value::<JaResponse>(raw_event)
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(
            event,
            PluginEvent::LegacyVideoRoomEvent(LegacyVideoRoomEvent::SubscriberAttached {
                room: JanusId::Uint(8146468.into()),
                id: JanusId::Uint(1337.into()),
                display: Some("Test Display Name".to_string()),
                jsep: Jsep {
                    jsep_type: JsepType::Offer,
                    sdp: "test_subscriber_sdp".to_string(),
                    trickle: None
                }
            })
        );
    }

    #[test]
    fn parse_slow_link() {
        let raw_event = json!({
            "janus": "event",
            "session_id": 7323526979899781u64,
            "sender": 7967725809069290u64,
            "plugindata": {
                "plugin": "janus.plugin.videoroom",
                "data": {
                    "videoroom": "slow_link"
                }
            }
        });
        let event: PluginEvent = serde_json::from_value::<JaResponse>(raw_event)
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(
            event,
            PluginEvent::LegacyVideoRoomEvent(LegacyVideoRoomEvent::SlowLink)
        );
    }

    #[test]
    fn parse_configured() {
        let raw_event = json!({
            "janus": "event",
            "session_id": 7323526979899781u64,
            "sender": 7967725809069290u64,
            "jsep": {
                "type": "answer",
                "sdp": "test_configured_sdp"
            },
            "plugindata": {
                "plugin": "janus.plugin.videoroom",
                "data": {
                    "videoroom": "event",
                    "room": 8146468u64,
                    "configured": "ok"
                }
            }
        });
        let event: PluginEvent = serde_json::from_value::<JaResponse>(raw_event)
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(
            event,
            PluginEvent::LegacyVideoRoomEvent(LegacyVideoRoomEvent::Configured {
                room: JanusId::Uint(8146468.into()),
                jsep: Some(Jsep {
                    jsep_type: JsepType::Answer,
                    sdp: "test_configured_sdp".to_string(),
                    trickle: None
                })
            })
        );
    }

    #[test]
    fn parse_unpublished() {
        let raw_event = json!({
            "janus": "event",
            "session_id": 7323526979899781u64,
            "sender": 7967725809069290u64,
            "plugindata": {
                "plugin": "janus.plugin.videoroom",
                "data": {
                    "videoroom": "event",
                    "room": 8146468u64,
                    "unpublished": 1337
                }
            }
        });
        let event: PluginEvent = serde_json::from_value::<JaResponse>(raw_event)
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(
            event,
            PluginEvent::LegacyVideoRoomEvent(LegacyVideoRoomEvent::Unpublished {
                room: JanusId::Uint(8146468.into()),
                unpublished: JanusId::Uint(1337.into())
            })
        );
    }

    #[test]
    fn parse_started() {
        let raw_event = json!({
            "janus": "event",
            "session_id": 7323526979899781u64,
            "sender": 7967725809069290u64,
            "plugindata": {
                "plugin": "janus.plugin.videoroom",
                "data": {
                    "videoroom": "event",
                    "room": 8146468u64,
                    "started": "ok"
                }
            }
        });
        let event: PluginEvent = serde_json::from_value::<JaResponse>(raw_event)
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(
            event,
            PluginEvent::LegacyVideoRoomEvent(LegacyVideoRoomEvent::SubscriberStarted {
                room: JanusId::Uint(8146468.into()),
                started: "ok".to_string()
            })
        );
    }

    #[test]
    fn parse_leaving() {
        let raw_event = json!({
            "janus": "event",
            "session_id": 7323526979899781u64,
            "sender": 7967725809069290u64,
            "plugindata": {
                "plugin": "janus.plugin.videoroom",
                "data": {
                    "videoroom": "event",
                    "room": 8146468u64,
                    "leaving": "ok",
                    "reason": "kicked"
                }
            }
        });
        let event: PluginEvent = serde_json::from_value::<JaResponse>(raw_event)
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(
            event,
            PluginEvent::LegacyVideoRoomEvent(LegacyVideoRoomEvent::Leaving {
                room: JanusId::Uint(8146468.into()),
                reason: "kicked".to_string()
            })
        );
    }

    #[test]
    fn parse_kicked() {
        let raw_event = json!({
            "janus": "event",
            "session_id": 7323526979899781u64,
            "sender": 7967725809069290u64,
            "plugindata": {
                "plugin": "janus.plugin.videoroom",
                "data": {
                    "videoroom": "event",
                    "room": 8146468u64,
                    "kicked": 1337
                }
            }
        });
        let event: PluginEvent = serde_json::from_value::<JaResponse>(raw_event)
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(
            event,
            PluginEvent::LegacyVideoRoomEvent(LegacyVideoRoomEvent::Kicked {
                room: JanusId::Uint(8146468.into()),
                participant: JanusId::Uint(1337.into())
            })
        );
    }

    #[test]
    fn parse_error() {
        let raw_event = json!({
            "janus": "event",
            "session_id": 7323526979899781u64,
            "sender": 7967725809069290u64,
            "plugindata": {
                "plugin": "janus.plugin.videoroom",
                "data": {
                    "error_code": 426,
                    "error": "No such room"
                }
            }
        });
        let event: PluginEvent = serde_json::from_value::<JaResponse>(raw_event)
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(
            event,
            PluginEvent::LegacyVideoRoomEvent(LegacyVideoRoomEvent::Error {
                error_code: 426,
                error: "No such room".to_string()
            })
        );
    }
}
