use super::common::AudioBridgeParticipant;
use crate::JanusId;
use jarust_interface::japrotocol::GenericEvent;
use jarust_interface::japrotocol::JaHandleEvent;
use jarust_interface::japrotocol::JaResponse;
use jarust_interface::japrotocol::Jsep;
use jarust_interface::japrotocol::PluginInnerData;
use jarust_interface::japrotocol::ResponseType;
use serde::Deserialize;
use serde_json::from_value;
use serde_json::Value;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
#[serde(tag = "audiobridge")]
enum AudioBridgeEventDto {
    #[serde(rename = "joined")]
    Joined(AudioBridgeJoinedEventDto),
    #[serde(rename = "left")]
    RoomLeft { id: JanusId, room: JanusId },
    #[serde(rename = "roomchanged")]
    RoomChanged {
        id: JanusId,
        room: JanusId,
        participants: Vec<AudioBridgeParticipant>,
    },
    #[serde(rename = "event")]
    Event(AudioBridgeEventEventType),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
#[serde(untagged)]
enum AudioBridgeJoinedEventDto {
    Room {
        id: JanusId,
        room: JanusId,
        participants: Vec<AudioBridgeParticipant>,
    },
    Participant {
        room: JanusId,
        participants: Vec<AudioBridgeParticipant>,
    },
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
#[serde(untagged)]
pub enum AudioBridgeEventEventType {
    Result {
        result: String,
    },
    ParticipantsUpdated {
        room: JanusId,
        participants: Vec<AudioBridgeParticipant>,
    },
    RoomMuteUpdated {
        room: JanusId,
        muted: bool,
    },
    ParticipantKicked {
        room: JanusId,
        kicked: JanusId,
    },
    ParticipantLeft {
        room: JanusId,
        leaving: JanusId,
    },
    KickedAll {
        room: JanusId,
        kicked_all: JanusId,
    },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PluginEvent {
    AudioBridgeEvent(AudioBridgeEvent),
    GenericEvent(GenericEvent),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum AudioBridgeEvent {
    Result {
        transaction: String,
        result: String,
    },
    ResultWithJsep {
        transaction: String,
        result: String,
        jsep: Jsep,
    },
    RoomJoinedWithJsep {
        id: JanusId,
        room: JanusId,
        participants: Vec<AudioBridgeParticipant>,
        jsep: Jsep,
    },
    RoomJoined {
        id: JanusId,
        room: JanusId,
        participants: Vec<AudioBridgeParticipant>,
    },
    RoomLeft {
        id: JanusId,
        room: JanusId,
    },
    RoomChanged {
        id: JanusId,
        room: JanusId,
        participants: Vec<AudioBridgeParticipant>,
    },
    RoomMuteUpdated {
        room: JanusId,
        muted: bool,
    },
    ParticipantsJoined {
        room: JanusId,
        participants: Vec<AudioBridgeParticipant>,
    },
    ParticipantsUpdated {
        room: JanusId,
        participants: Vec<AudioBridgeParticipant>,
    },
    ParticipantKicked {
        room: JanusId,
        kicked: JanusId,
    },
    ParticipantLeft {
        room: JanusId,
        leaving: JanusId,
    },
    KickedAll {
        room: JanusId,
        kicked_all: JanusId,
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
                let audiobridge_event = match plugin_data.data {
                    PluginInnerData::Error { error_code, error } => {
                        AudioBridgeEvent::Error { error_code, error }
                    }
                    PluginInnerData::Data(data) => {
                        match from_value::<AudioBridgeEventDto>(data.clone()) {
                            Ok(event) => match event {
                                AudioBridgeEventDto::Event(AudioBridgeEventEventType::Result {
                                    result,
                                }) => match (value.transaction, value.jsep) {
                                    (None, _) => return Err(Self::Error::IncompletePacket),
                                    (Some(transaction), Some(jsep)) => {
                                        AudioBridgeEvent::ResultWithJsep {
                                            transaction,
                                            result,
                                            jsep,
                                        }
                                    }
                                    (Some(transaction), None) => AudioBridgeEvent::Result {
                                        transaction,
                                        result,
                                    },
                                },
                                AudioBridgeEventDto::Joined(AudioBridgeJoinedEventDto::Room {
                                    id,
                                    room,
                                    participants,
                                }) => match value.jsep {
                                    Some(jsep) => AudioBridgeEvent::RoomJoinedWithJsep {
                                        id,
                                        room,
                                        participants,
                                        jsep,
                                    },
                                    None => AudioBridgeEvent::RoomJoined {
                                        id,
                                        room,
                                        participants,
                                    },
                                },
                                AudioBridgeEventDto::Joined(
                                    AudioBridgeJoinedEventDto::Participant { room, participants },
                                ) => AudioBridgeEvent::ParticipantsJoined { room, participants },
                                AudioBridgeEventDto::RoomLeft { id, room } => {
                                    AudioBridgeEvent::RoomLeft { id, room }
                                }
                                AudioBridgeEventDto::RoomChanged {
                                    id,
                                    room,
                                    participants,
                                } => AudioBridgeEvent::RoomChanged {
                                    id,
                                    room,
                                    participants,
                                },
                                AudioBridgeEventDto::Event(
                                    AudioBridgeEventEventType::ParticipantsUpdated {
                                        room,
                                        participants,
                                    },
                                ) => AudioBridgeEvent::ParticipantsUpdated { room, participants },
                                AudioBridgeEventDto::Event(
                                    AudioBridgeEventEventType::RoomMuteUpdated { room, muted },
                                ) => AudioBridgeEvent::RoomMuteUpdated { room, muted },
                                AudioBridgeEventDto::Event(
                                    AudioBridgeEventEventType::ParticipantKicked { room, kicked },
                                ) => AudioBridgeEvent::ParticipantKicked { room, kicked },
                                AudioBridgeEventDto::Event(
                                    AudioBridgeEventEventType::ParticipantLeft { room, leaving },
                                ) => AudioBridgeEvent::ParticipantLeft { room, leaving },
                                AudioBridgeEventDto::Event(
                                    AudioBridgeEventEventType::KickedAll { room, kicked_all },
                                ) => AudioBridgeEvent::KickedAll { room, kicked_all },
                            },
                            Err(_) => AudioBridgeEvent::Other(data),
                        }
                    }
                };
                Ok(PluginEvent::AudioBridgeEvent(audiobridge_event))
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
    use crate::audio_bridge::common::AudioBridgeParticipant;
    use crate::audio_bridge::events::AudioBridgeEvent;
    use crate::JanusId;
    use jarust_interface::japrotocol::JaHandleEvent;
    use jarust_interface::japrotocol::JaResponse;
    use jarust_interface::japrotocol::Jsep;
    use jarust_interface::japrotocol::JsepType;
    use jarust_interface::japrotocol::PluginData;
    use jarust_interface::japrotocol::PluginInnerData;
    use jarust_interface::japrotocol::ResponseType;
    use serde_json::json;

    #[test]
    fn it_parse_room_joined() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.audiobridge".to_string(),
                    data: PluginInnerData::Data(json!({
                        "audiobridge": "joined",
                        "room": 684657u64,
                        "id": 751378u64,
                        "participants": []
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
            PluginEvent::AudioBridgeEvent(AudioBridgeEvent::RoomJoined {
                id: JanusId::Uint(751378u64.try_into().unwrap()),
                room: JanusId::Uint(684657u64.try_into().unwrap()),
                participants: vec![],
            })
        );
    }

    #[test]
    fn it_parse_room_joined_with_jsep_event() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.audiobridge".to_string(),
                    data: PluginInnerData::Data(json!({
                        "audiobridge": "joined",
                        "room": 684657u64,
                        "id": 751378u64,
                        "participants": []
                    })),
                },
            }),
            jsep: Some(Jsep {
                jsep_type: JsepType::Answer,
                trickle: Some(false),
                sdp: "test_sdp".to_string(),
            }),
            transaction: None,
            session_id: None,
            sender: None,
        };
        let event: PluginEvent = rsp.try_into().unwrap();
        assert_eq!(
            event,
            PluginEvent::AudioBridgeEvent(AudioBridgeEvent::RoomJoinedWithJsep {
                id: JanusId::Uint(751378u64.try_into().unwrap()),
                room: JanusId::Uint(684657u64.try_into().unwrap()),
                participants: vec![],
                jsep: Jsep {
                    jsep_type: JsepType::Answer,
                    trickle: Some(false),
                    sdp: "test_sdp".to_string(),
                },
            })
        );
    }

    #[test]
    fn it_parse_room_left() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.audiobridge".to_string(),
                    data: PluginInnerData::Data(json!({
                        "audiobridge": "left",
                        "room": 684657u64,
                        "id": 751378u64
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
            PluginEvent::AudioBridgeEvent(AudioBridgeEvent::RoomLeft {
                id: JanusId::Uint(751378.try_into().unwrap()),
                room: JanusId::Uint(684657.try_into().unwrap()),
            })
        );
    }

    #[test]
    fn it_parse_room_changed() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.audiobridge".to_string(),
                    data: PluginInnerData::Data(json!({
                        "audiobridge": "roomchanged",
                        "room": 61682u64,
                        "id": 38626u64,
                        "participants": []
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
            PluginEvent::AudioBridgeEvent(AudioBridgeEvent::RoomChanged {
                id: JanusId::Uint(38626.try_into().unwrap()),
                room: JanusId::Uint(61682.try_into().unwrap()),
                participants: vec![],
            })
        );
    }

    #[test]
    fn it_parse_participants_updated() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.audiobridge".to_string(),
                    data: PluginInnerData::Data(json!({
                        "audiobridge": "event",
                        "room": 6613848040355181645u64,
                        "participants": [
                            {
                                "id": 4975437903264518u64,
                                "setup": false,
                                "muted": false
                            }
                        ]
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
            PluginEvent::AudioBridgeEvent(AudioBridgeEvent::ParticipantsUpdated {
                room: JanusId::Uint(6613848040355181645.try_into().unwrap()),
                participants: vec![AudioBridgeParticipant {
                    id: JanusId::Uint(4975437903264518u64.try_into().unwrap()),
                    setup: false,
                    muted: false,
                    display: None,
                    suspended: None,
                    talking: None,
                    spatial_position: None
                }]
            })
        );
    }

    #[test]
    fn it_parse_room_mute_updated() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.audiobridge".to_string(),
                    data: PluginInnerData::Data(json!({
                        "audiobridge": "event",
                        "room": 6613848040355181645u64,
                        "muted": true
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
            PluginEvent::AudioBridgeEvent(AudioBridgeEvent::RoomMuteUpdated {
                room: JanusId::Uint(6613848040355181645.try_into().unwrap()),
                muted: true
            })
        );
    }

    #[test]
    fn it_parse_participant_kicked() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.audiobridge".to_string(),
                    data: PluginInnerData::Data(json!({
                        "audiobridge": "event",
                        "room": 6613848040355181645u64,
                        "kicked": 4975437903264518u64
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
            PluginEvent::AudioBridgeEvent(AudioBridgeEvent::ParticipantKicked {
                room: JanusId::Uint(6613848040355181645.try_into().unwrap()),
                kicked: JanusId::Uint(4975437903264518u64.try_into().unwrap())
            })
        );
    }

    #[test]
    fn it_parse_participant_left() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.audiobridge".to_string(),
                    data: PluginInnerData::Data(json!({
                        "audiobridge": "event",
                        "room": 6613848040355181645u64,
                        "leaving": 4975437903264518u64
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
            PluginEvent::AudioBridgeEvent(AudioBridgeEvent::ParticipantLeft {
                room: JanusId::Uint(6613848040355181645.try_into().unwrap()),
                leaving: JanusId::Uint(4975437903264518u64.try_into().unwrap())
            })
        );
    }

    #[test]
    fn it_parse_kicked_all_event() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.audiobridge".to_string(),
                    data: PluginInnerData::Data(json!({
                        "audiobridge": "event",
                        "room": 6613848040355181645u64,
                        "kicked_all": 4975437903264518u64
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
            PluginEvent::AudioBridgeEvent(AudioBridgeEvent::KickedAll {
                room: JanusId::Uint(6613848040355181645.try_into().unwrap()),
                kicked_all: JanusId::Uint(4975437903264518u64.try_into().unwrap())
            })
        );
    }

    #[test]
    fn it_parse_result_event_when_contains_transaction() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.audiobridge".to_string(),
                    data: PluginInnerData::Data(json!({
                        "audiobridge": "event",
                        "result": "ok"
                    })),
                },
            }),
            jsep: None,
            transaction: Some("test_transaction".to_string()),
            session_id: None,
            sender: None,
        };
        let event: PluginEvent = rsp.try_into().unwrap();
        assert_eq!(
            event,
            PluginEvent::AudioBridgeEvent(AudioBridgeEvent::Result {
                transaction: "test_transaction".to_string(),
                result: "ok".to_string()
            })
        );
    }

    #[test]
    fn it_parse_result_event_when_contains_transaction_and_jsep() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.audiobridge".to_string(),
                    data: PluginInnerData::Data(json!({
                        "audiobridge": "event",
                        "result": "ok"
                    })),
                },
            }),
            jsep: Some(Jsep {
                jsep_type: JsepType::Answer,
                trickle: Some(false),
                sdp: "test_sdp".to_string(),
            }),
            transaction: Some("test_transaction".to_string()),
            session_id: None,
            sender: None,
        };
        let event: PluginEvent = rsp.try_into().unwrap();
        assert_eq!(
            event,
            PluginEvent::AudioBridgeEvent(AudioBridgeEvent::ResultWithJsep {
                transaction: "test_transaction".to_string(),
                result: "ok".to_string(),
                jsep: Jsep {
                    jsep_type: JsepType::Answer,
                    trickle: Some(false),
                    sdp: "test_sdp".to_string(),
                }
            })
        );
    }

    #[test]
    fn it_parse_return_err_when_parsing_result_without_transaction_and_without_jsep() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.audiobridge".to_string(),
                    data: PluginInnerData::Data(json!({
                        "audiobridge": "event",
                        "result": "ok"
                    })),
                },
            }),
            jsep: None,
            transaction: None,
            session_id: None,
            sender: None,
        };
        let event: Result<PluginEvent, jarust_interface::Error> = rsp.try_into();
        matches!(event, Err(jarust_interface::Error::IncompletePacket));
    }

    #[test]
    fn it_parse_return_err_when_parsing_result_with_jsep_and_without_transaction() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.audiobridge".to_string(),
                    data: PluginInnerData::Data(json!({
                        "audiobridge": "event",
                        "result": "ok"
                    })),
                },
            }),
            jsep: Some(Jsep {
                jsep_type: JsepType::Answer,
                trickle: Some(false),
                sdp: "test_sdp".to_string(),
            }),
            transaction: None,
            session_id: None,
            sender: None,
        };
        let event: Result<PluginEvent, jarust_interface::Error> = rsp.try_into();
        matches!(event, Err(jarust_interface::Error::IncompletePacket));
    }

    #[test]
    fn it_parse_unsupported_event_as_other() {
        let rsp = JaResponse {
            janus: ResponseType::Event(JaHandleEvent::PluginEvent {
                plugin_data: PluginData {
                    plugin: "janus.plugin.audiobridge".to_string(),
                    data: PluginInnerData::Data(json!({
                        "audiobridge": "jarust_rocks",
                        "room": 6613848040355181645u64,
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
            PluginEvent::AudioBridgeEvent(AudioBridgeEvent::Other(json!({
                "audiobridge": "jarust_rocks",
                "room": 6613848040355181645u64,
                "jarust": "rocks"
            })))
        );
    }
}
