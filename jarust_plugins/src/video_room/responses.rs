use crate::JanusId;
use serde::Deserialize;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct Room {
    /// unique numeric ID
    pub room: JanusId,
    /// name of the room
    pub description: String,
    /// whether a PIN is required to join this room
    pub pin_required: bool,
    /// whether this room is 'private' (as in hidden) or not
    pub is_private: bool,
    /// how many publishers can actually publish via WebRTC at the same time
    pub max_publishers: u64,
    /// bitrate cap that should be forced (via REMB) on all publishers by default
    pub bitrate: u64,
    /// whether the above cap should act as a limit to dynamic bitrate changes by publishers
    pub bitrate_cap: Option<bool>,
    /// how often a keyframe request is sent via PLI/FIR to active publishers
    pub fir_freq: u64,
    /// whether subscriptions in this room require a private_id
    pub require_pvtid: bool,
    /// whether end-to-end encrypted publishers are required
    pub require_e2ee: bool,
    /// whether a dummy publisher exists for placeholder subscriptions
    pub dummy_publisher: bool,
    /// whether an event is sent to notify all participants if a new participant joins the room
    pub notify_joining: bool,
    /// comma separated list of allowed audio codecs
    pub audiocodec: String,
    /// comma separated list of allowed video codecs
    pub videocodec: String,
    /// whether inband FEC must be negotiated (note: only available for Opus)
    pub opus_fec: Option<bool>,
    /// whether DTX must be negotiated (note: only available for Opus)
    pub opus_dtx: Option<bool>,
    /// whether the room is being recorded
    pub record: bool,
    /// if recording, the path where the .mjr files are being saved
    pub rec_dir: Option<String>,
    /// whether the room recording state can only be changed providing the secret
    pub lock_record: bool,
    /// count of the participants (publishers, active or not; not subscribers)
    pub num_participants: u64,
    /// whether the ssrc-audio-level extension must be negotiated or not for new publishers
    pub audiolevel_ext: bool,
    /// whether to emit event to other users about audiolevel
    pub audiolevel_event: bool,
    /// number of packets with audio level for checkup (optional, only if audiolevel_event is true)
    pub audio_active_packets: Option<u64>,
    /// average audio level (optional, only if audiolevel_event is true)
    pub audio_level_average: Option<u64>,
    /// whether the video-orientation extension must be negotiated or not for new publishers
    pub videoorient_ext: bool,
    /// whether the playout-delay extension must be negotiated or not for new publishers
    pub playoutdelay_ext: bool,
    /// whether the transport wide cc extension must be negotiated or not for new publishers
    pub transport_wide_cc_ext: bool,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct VideoRoomParticipant {
    /// unique numeric ID of the participant
    pub id: JanusId,
    /// display name of the participant, if any
    pub display: Option<String>,
    /// whether user is an active publisher in the room
    pub publisher: bool,
    /// whether user is talking or not (only if audio levels are used)
    pub talking: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct Publisher {
    /// unique ID of active publisher
    pub id: JanusId,
    /// display name of active publisher
    pub display: Option<String>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct Stream {
    /// type of published stream (audio|video|data)
    #[serde(rename = "type")]
    pub media_type: String,

    /// unique mindex of published stream
    pub mindex: u64,

    /// unique mid of published stream
    pub mid: u64,

    /// if true, it means this stream is currently inactive/disabled
    /// (and so codec, description, etc. will be missing)
    pub disabled: bool,

    /// codec used for published stream
    pub codec: Option<String>,

    /// text description of published stream, if any
    pub description: Option<String>,

    /// true if this stream audio has been moderated for this participant
    pub moderated: Option<bool>,

    /// true if published stream uses simulcast
    pub simulcast: Option<bool>,

    /// true if published stream uses SVC (VP9 and AV1 only)
    pub svc: Option<bool>,

    /// whether the publisher stream has audio activity or not (only if audio levels are used)
    pub talking: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct Attendee {
    /// unique ID of the attendee
    pub id: JanusId,

    /// display name of the attendee, if any
    pub display: Option<String>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct RtpForwarderPublisher {
    pub publisher_id: JanusId,
    pub forwarders: Vec<RtpForwarderStream>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct RtpForwarderStream {
    /// unique numeric ID assigned to this forwarder
    pub stream_id: u64,
    /// type of media (audio|video|data)
    #[serde(rename = "type")]
    pub media_type: String,
    /// host this forwarder is streaming to, same as request if not resolved
    pub host: String,
    /// port this forwarder is streaming to, same as request if configured
    pub port: u64,
    /// local port this forwarder is using to get RTCP feedback, if any
    pub local_rtcp_port: Option<u64>,
    /// remote port this forwarder is getting RTCP feedback from, if any
    pub remote_rtmp_port: Option<u64>,
    /// SSRC this forwarder is using, same as request if configured
    pub ssrc: Option<String>,
    /// payload type this forwarder is using, same as request if configured
    pub pt: Option<String>,
    /// video substream this video forwarder is relaying
    pub substream: Option<u64>,
    /// whether the RTP stream is encrypted (not used for data)
    pub strp: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct AttachedStream {
    /// unique mindex of published stream
    pub mindex: u64,

    /// unique mid of published stream
    pub mid: String,

    /// type of published stream (audio|video|data)
    #[serde(rename = "type")]
    pub media_type: String,

    /// whether this stream is currently active
    pub active: bool,

    /// unique ID of the publisher originating this stream
    pub feed_id: JanusId,

    /// unique mid of this publisher's stream
    pub feed_mid: String,

    /// display name of this publisher, if any
    pub feed_display: Option<String>,

    /// whether we configured the stream to relay media
    pub send: bool,

    /// codec used by this stream
    pub codec: String,

    /// in case H.264 is used by the stream, the negotiated profile
    #[serde(rename = "h264-profile")]
    pub h264_profile: Option<String>,

    /// in case VP9 is used by the stream, the negotiated profile
    #[serde(rename = "vp9-profile")]
    pub vp9_profile: Option<String>,

    /// whether this stream is ready to start sending media (will be false at the beginning)
    pub ready: bool,

    /// if this is a data channel stream, the number of data channel subscriptions
    pub sources: Option<i64>,

    /// if this is a data channel stream, an array containing the IDs of participants we've subscribed to
    pub source_ids: Option<Vec<JanusId>>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Deserialize)]
pub struct ConfiguredStream {
    /// type of published stream (audio|video|data)
    #[serde(rename = "type")]
    pub media_type: String,

    /// unique mindex of published stream
    pub mindex: u64,

    /// unique mid of published stream
    pub mid: String,

    /// if the stream is disabled, all fields below will be missing
    #[serde(default)]
    pub disabled: bool,

    /// codec used by this stream
    #[serde(default)]
    pub codec: String,

    /// audio: opus codec: whether the stream has stereo
    #[serde(default)]
    pub stereo: bool,

    /// audio: opus codec: whether OPUS forward error correction is enabled
    #[serde(default)]
    pub fec: bool,

    /// audio: opus codec: whether OPUS discontinuous transmission is enabled
    #[serde(default)]
    pub dtx: bool,

    /// video: in case H.264 is used by the stream, the negotiated profile
    pub h264_profile: Option<String>,

    /// video: in case VP9 is used by the stream, the negotiated profile
    pub vp9_profile: Option<String>,

    /// video: true if this stream audio has been moderated for this participant
    #[serde(default)]
    pub moderated: bool,

    /// video: true if stream uses simulcast
    #[serde(default)]
    pub simulcast: bool,

    /// video: true if published stream #1 uses SVC (VP9 and AV1 only)
    #[serde(default)]
    pub svc: bool,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct VideoRoomCreatedRsp {
    pub room: JanusId,
    pub permanent: bool,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct VideoRoomListRoomsRsp {
    pub list: Vec<Room>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct ListParticipantsRsp {
    pub room: JanusId,
    pub participants: Vec<VideoRoomParticipant>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct VideoRoomDestroyedRsp {
    pub room: JanusId,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct VideoRoomEditedRsp {
    pub room: JanusId,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct VideoRoomExistsRsp {
    pub room: JanusId,
    pub exists: bool,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct VideoRoomAccessRsp {
    pub room: JanusId,
    #[serde(default = "Vec::default")]
    pub allowed: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct VideoRoomListForwardersRsp {
    /// unique ID of the room
    pub room: JanusId,

    /// Array of publishers with RTP forwarders
    pub publisher: Vec<RtpForwarderPublisher>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct VideoRoomRtpForwardRsp {
    /// unique ID, same as request
    pub room: JanusId,

    /// unique ID, same as request
    pub publisher_id: JanusId,

    pub forwarders: Vec<RtpForwarderStream>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub struct VideoRoomStopRtpForwardRsp {
    /// unique ID, same as request
    pub room: JanusId,

    /// unique ID, same as request
    pub publisher_id: JanusId,

    /// unique numeric ID, same as request
    pub stream_id: u64,
}

#[cfg(test)]
mod tests {
    use crate::video_room::responses::AttachedStream;
    use crate::JanusId;

    #[test]
    fn parse_attached_stream() {
        let source = "{
            \"type\": \"audio\",
            \"active\": true,
            \"mindex\": 0,
            \"mid\": \"0\",
            \"ready\": false,
            \"send\": true,
            \"feed_id\": \"a8cabfaa-da33-4627-9938-57c39ecd94d8\",
            \"feed_mid\": \"0\",
            \"codec\": \"opus\"
        }";

        let dut: AttachedStream = serde_json::from_str(source).unwrap();

        assert_eq!(
            dut,
            AttachedStream {
                media_type: "audio".to_string(),
                active: true,
                mindex: 0,
                mid: "0".to_string(),
                ready: false,
                send: true,
                feed_id: JanusId::String("a8cabfaa-da33-4627-9938-57c39ecd94d8".to_string()),
                feed_mid: "0".to_string(),
                codec: "opus".to_string(),
                feed_display: None,
                h264_profile: None,
                vp9_profile: None,
                sources: None,
                source_ids: None,
            }
        )
    }
}
