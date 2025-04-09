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
        private_id: u64,
        description: Option<String>,
        publishers: Vec<LegacyVideoRoomPublisher>,
    },
    #[serde(rename = "slow_link")]
    SlowLink,
    #[serde(rename = "event")]
    Event(InnerLegacyVideoRoomEvent),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
#[serde(untagged)]
enum InnerLegacyVideoRoomEvent {
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
        private_id: u64,
        publishers: Vec<LegacyVideoRoomPublisher>,
        jsep: Option<Jsep>,
    },
    SlowLink,
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
                                LegacyVideoRoomEventDto::SlowLink => LegacyVideoRoomEvent::SlowLink,
                                LegacyVideoRoomEventDto::Event(event) => match event {
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
