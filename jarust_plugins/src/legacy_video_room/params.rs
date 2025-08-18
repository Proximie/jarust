use crate::JanusId;
use serde::Serialize;
use std::collections::HashSet;

make_dto!(
    LegacyVideoRoomCreateParams,
    optional {
        /// Can be configured in plugin settings. If set, rooms can be created via API only if this key is provided in the request
        admin_key: String,
        /// Room ID, chosen by plugin if missing
        room: JanusId,
        /// pretty name of the room
        description: String,
        /// whether the room should appear in a list request
        is_private: bool,
        /// array of string tokens users can use to join this room
        allowed: Vec<String>,
        /// password required to edit/destroy the room
        secret: String,
        /// password required to join the room
        pin: String,
        /// whether subscriptions are required to provide a valid private_id to associate with a publisher, default=false
        require_pvtid: bool,
        /// whether access to the room requires signed tokens; default=false, only works if signed tokens are used in the core as well
        signed_tokens: bool,
        /// max video bitrate for senders (e.g., 128000)
        bitrate: u64,
        /// whether the above cap should act as a limit to dynamic bitrate changes by publishers, default=false
        bitrate_cap: bool,
        /// send a FIR to publishers every fir_freq seconds (0=disable)
        fir_freq: u64,
        /// max number of concurrent senders (e.g., 6 for a video conference or 1 for a webinar, default=3)
        publishers: u64,
        /// audio codec to force on publishers, default=opus
        /// can be a comma separated list in order of preference, e.g., `opus,pcmu`
        /// opus|g722|pcmu|pcma|isac32|isac16
        audiocodec: LegacyVideoRoomAudioCodecList,
        /// video codec to force on publishers, default=vp8
        /// can be a comma separated list in order of preference, e.g., `vp9,vp8,h264`
        /// vp8|vp9|h264|av1|h265
        videocodec: LegacyVideoRoomVideoCodecList,
        /// VP9-specific profile to prefer (e.g., "2" for "profile-id=2")
        vp9_profile: String,
        /// H.264-specific profile to prefer (e.g., "42e01f" for "profile-level-id=42e01f")
        h264_profile: String,
        /// whether inband FEC must be negotiated; only works for Opus, default=true
        opus_fec: bool,
        /// whether DTX must be negotiated; only works for Opus, default=false
        opus_dtx: bool,
        /// whether the ssrc-audio-level RTP extension must be negotiated for new joins, default=true
        audiolevel_ext: bool,
        /// whether to emit event to other users or not
        audiolevel_event: bool,
        /// number of packets with audio level (default=100, 2 seconds)
        audio_active_packets: u64,
        /// average value of audio level (127=muted, 0='too loud', default=25)
        audio_level_average: u64,
        /// whether the video-orientation RTP extension must be negotiated/used or not for new publishers, default=true
        videoorient_ext: bool,
        /// whether the playout-delay RTP extension must be negotiated/used or not for new publishers, default=true
        playoutdelay_ext: bool,
        /// whether the transport wide CC RTP extension must be negotiated/used or not for new publishers, default=true
        transport_wide_cc_ext: bool,
        /// whether to record the room or not, default=false
        record: bool,
        /// folder where recordings should be stored, when enabled
        rec_dir: String,
        /// whether recording can only be started/stopped if the secret is provided, or using the global enable_recording request, default=false
        lock_record: bool,
        /// whether the room should be saved in the config file, default=false
        permanent: bool,
        /// optional, whether to notify all participants when a new participant joins the room. default=false
        /// The Videoroom plugin by design only notifies new feeds (publishers), and enabling this may result in extra notification traffic.
        /// This flag is particularly useful when enabled with `require_pvtid` for admin to manage listening-only participants.
        notify_joining: bool,
        /// whether all participants are required to publish and subscribe using end-to-end media encryption, e.g., via Insertable Streams; default=false
        require_e2ee: bool,
        /// whether a dummy publisher should be created in this room, with one separate m-line for each codec supported in the room;
        /// this is useful when there's a need to create subscriptions with placeholders for some or all m-lines, even when they aren't used yet; default=false
        dummy_publisher: bool,
        /// in case `dummy_publisher` is set to `true`, array of codecs to offer, optionally with a fmtp attribute to match (codec/fmtp properties).
        /// If not provided, all codecs enabled in the room are offered, with no fmtp. Notice that the fmtp is parsed, and only a few codecs are supported.
        dummy_streams: bool,
    }
);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LegacyVideoRoomAudioCodec {
    OPUS,
    G722,
    PCMU,
    PCMA,
    ISAC32,
    ISAC16,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct LegacyVideoRoomAudioCodecList {
    pub codecs: Vec<LegacyVideoRoomAudioCodec>,
}

impl LegacyVideoRoomAudioCodecList {
    pub fn new(codecs: Vec<LegacyVideoRoomAudioCodec>) -> Self {
        let codecs = codecs
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        Self { codecs }
    }
}

impl Serialize for LegacyVideoRoomAudioCodecList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let codecs = self
            .codecs
            .iter()
            .flat_map(|codec| match serde_json::to_string(codec) {
                Ok(codec) => Some(codec.trim_matches('"').to_string()),
                Err(_) => None,
            })
            .collect::<Vec<_>>()
            .join(",");
        let state = serializer.serialize_str(&codecs)?;
        Ok(state)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LegacyVideoRoomVideoCodec {
    VP8,
    VP9,
    H264,
    AV1,
    H265,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct LegacyVideoRoomVideoCodecList {
    pub codecs: Vec<LegacyVideoRoomVideoCodec>,
}

impl LegacyVideoRoomVideoCodecList {
    pub fn new(codecs: Vec<LegacyVideoRoomVideoCodec>) -> Self {
        let codecs = codecs
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        Self { codecs }
    }
}

impl Serialize for LegacyVideoRoomVideoCodecList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let codecs = self
            .codecs
            .iter()
            .flat_map(|codec| match serde_json::to_string(codec) {
                Ok(codec) => Some(codec.trim_matches('"').to_string()),
                Err(_) => None,
            })
            .collect::<Vec<_>>()
            .join(",");
        let state = serializer.serialize_str(&codecs)?;
        Ok(state)
    }
}

make_dto!(LegacyVideoRoomExistsParams, required { room: JanusId });

make_dto!(
    LegacyVideoRoomKickParams,
    required {
        room: JanusId,
        id: JanusId
    },
    optional {
        /// room secret, mandatory if configured
        secret: String
    }
);

make_dto!(
    LegacyVideoRoomPublisherJoinParams,
    required {
        /// unique ID to register for the publisher;
        /// optional, will be chosen by the plugin if missing
        room: JanusId
    },
    optional {
        /// unique ID to register for the publisher;
        /// optional, will be chosen by the plugin if missing
        id: JanusId,
        /// display name for the publisher
        display: String,
        /// invitation token, in case the room has an ACL
        token: String
    }
);

make_dto!(
    LegacyVideoRoomPublisherConfigureParams,
    optional {
        /// depending on whether or not audio should be relayed; true by default
        audio: bool,
        /// depending on whether or not video should be relayed; true by default
        video: bool,
        /// depending on whether or not data should be relayed; true by default
        data: bool,
        /// bitrate cap to return via REMB;
        /// overrides the global room value if present (unless `bitrate_cap` is set)
        bitrate: u64,
        /// whether we should send this publisher a keyframe request
        keyframe: bool,
        /// whether this publisher should be recorded or not
        record: bool,
        /// if recording, the base path/file to use for the recording files
        filename: String,
        /// new display name to use in the room
        display: String,
        /// new `audio_active_packets` to overwrite in the room one
        audio_active_packets: u64,
        /// new `audio_level_average` to overwrite the room one
        audio_level_average: u64,
        /// minimum delay to enforce via the playout-delay RTP extension, in blocks of 10ms
        min_delay: u64,
        /// maximum delay to enforce via the playout-delay RTP extension, in blocks of 10ms
        max_delay: u64,
        /// video codec to prefer among the negotiated ones
        videocodec: LegacyVideoRoomVideoCodec
    }
);

make_dto!(
    LegacyVideoRoomPublisherJoinAndConfigureParams,
    required {
        #[serde(flatten)]
        join_params: LegacyVideoRoomPublisherJoinParams,
        #[serde(flatten)]
        configure_params: LegacyVideoRoomPublisherConfigureParams
    }
);

make_dto!(
    LegacyVideoRoomSubscriberJoinParams,
    required {
        /// unique ID of the room to subscribe in
        room: JanusId,
        /// unique ID of the publisher to subscribe to
        feed: JanusId,
    },
    optional {
        /// unique ID of the publisher that originated this request; optional, unless mandated by the room configuration
        private_id: u64,
        /// depending on whether or not the PeerConnection should be automatically closed when the publisher leaves; true by default
        close_pc: bool,
        /// depending on whether or not audio should be relayed; true by default
        audio: bool,
        /// depending on whether or not video should be relayed; true by default
        video: bool,
        /// depending on whether or not data should be relayed; true by default
        data: bool,
        /// whether or not audio should be negotiated; true by default if the publisher has audio
        offer_audio: bool,
        /// whether or not video should be negotiated; true by default if the publisher has video
        offer_video: bool,
        /// whether or not datachannels should be negotiated; true by default if the publisher has datachannels
        offer_data: bool,
        /// substream to receive (0-2), in case simulcasting is enabled; optional
        substream: u8,
        /// temporal layers to receive (0-2), in case simulcasting is enabled; optional
        temporal: u8,
        /// How much time (in us, default 250000) without receiving packets will make us drop to the substream below
        fallback: u64,
        /// spatial layer to receive (0-2), in case VP9-SVC is enabled; optional
        spatial_layer: u8,
        /// temporal layers to receive (0-2), in case VP9-SVC is enabled; optional
        temporal_layer: u8,
    }
);

make_dto!(
    LegacyVideoRoomSubscriberConfigureParams,
    optional {
        /// depending on whether audio should be relayed or not
        audio: bool,
        /// depending on whether video should be relayed or not
        video: bool,
        /// depending on whether datachannel messages should be relayed or not
        data: bool,
        /// substream to receive (0-2), in case simulcasting is enabled
        substream: u8,
        /// temporal layers to receive (0-2), in case simulcasting is enabled
        temporal: u8,
        /// How much time (in us, default 250000) without receiving packets will make us drop to the substream below
        fallback: u64,
        /// spatial layer to receive (0-2), in case VP9-SVC is enabled
        spatial_layer: u8,
        /// temporal layers to receive (0-2), in case VP9-SVC is enabled
        temporal_layer: u8,
        /// overrides the room audio_level_average for this user
        audio_level_average: u64,
        /// overrides the room audio_active_packets for this user
        audio_active_packets: u64,
        /// minimum delay to enforce via the playout-delay RTP extension, in blocks of 10ms
        min_delay: u64,
        /// maximum delay to enforce via the playout-delay RTP extension, in blocks of 10ms
        max_delay: u64,
    }
);
