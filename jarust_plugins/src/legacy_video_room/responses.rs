use crate::JanusId;
use serde::Deserialize;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct LegacyVideoRoomCreatedRsp {
    pub room: JanusId,
    pub permanent: bool,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct LegacyVideoRoomExistsRsp {
    pub room: JanusId,
    pub exists: bool,
}
