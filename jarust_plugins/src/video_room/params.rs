use crate::JanusId;
use serde::Serialize;
use std::collections::HashSet;

make_dto!(
    VideoRoomCreateParams,
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
        audiocodec: VideoRoomAudioCodecList,
        /// video codec to force on publishers, default=vp8
        /// can be a comma separated list in order of preference, e.g., `vp9,vp8,h264`
        /// vp8|vp9|h264|av1|h265
        videocodec: VideoRoomVideoCodecList,
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
        record_dir: String,
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
pub enum VideoRoomAudioCodec {
    OPUS,
    G722,
    PCMU,
    PCMA,
    ISAC32,
    ISAC16,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct VideoRoomAudioCodecList {
    pub codecs: Vec<VideoRoomAudioCodec>,
}

impl VideoRoomAudioCodecList {
    pub fn new(codecs: Vec<VideoRoomAudioCodec>) -> Self {
        let codecs = codecs
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        Self { codecs }
    }
}

impl Serialize for VideoRoomAudioCodecList {
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
pub enum VideoRoomVideoCodec {
    VP8,
    VP9,
    H264,
    AV1,
    H265,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct VideoRoomVideoCodecList {
    pub codecs: Vec<VideoRoomVideoCodec>,
}

impl VideoRoomVideoCodecList {
    pub fn new(codecs: Vec<VideoRoomVideoCodec>) -> Self {
        let codecs = codecs
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        Self { codecs }
    }
}

impl Serialize for VideoRoomVideoCodecList {
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

make_dto!(
    VideoRoomEditParams,
    required { room: JanusId },
    optional {
        /// room secret, mandatory if configured
        secret: String,
        /// new pretty name of the room
        new_description: String,
        /// whether the room should appear in a list request
        new_is_private: bool,
        /// new password required to edit/destroy the room
        new_secret: String,
        /// new PIN required to join the room, PIN will be removed if set to an empty string
        new_pin: String,
        /// whether the room should require `private_id` from subscribers
        new_require_pvtid: bool,
        /// new bitrate cap to force on all publishers (except those with custom overrides)
        new_bitrate: u64,
        /// new period for regular PLI keyframe requests to publishers
        new_fir_freq: u64,
        /// new cap on the number of concurrent active WebRTC publishers
        new_publishers: u64,
        /// whether recording state can only be changed when providing the room secret
        new_lock_record: bool,
        /// the new path where the next .mjr files should be saved
        new_rec_dir: String,
        /// whether the room should be also removed from the config file, default=false
        permanent: bool,
    }
);

make_dto!(
    VideoRoomDestroyParams,
    required { room: JanusId },
    optional {
        /// room secret, mandatory if configured
        secret: String,
        /// whether the room should be also removed from the config file, default=false
        permanent: bool
    }
);

make_dto!(
    VideoRoomAllowedParams,
    required {
        room: JanusId,
        action: VideoRoomAllowedAction,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        allowed: Vec<String>
    },
    optional {
        /// room secret, mandatory if configured
        secret: String
    }
);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoRoomAllowedAction {
    Enable,
    Disable,
    Add,
    Remove,
}

make_dto!(
    VideoRoomKickParams,
    required {
        room: JanusId,
        participant: JanusId
    },
    optional {
        /// room secret, mandatory if configured
        secret: String
    }
);

make_dto!(
    VideoRoomModerateParams,
    required {
        room: JanusId,
        participant: JanusId,
        m_line: u64
    },
    optional {
        /// room secret, mandatory if configured
        secret: String
    }
);

make_dto!(
    VideoRoomPublisherJoinParams,
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
    VideoRoomPublishParams,
    optional {
        /// audio codec to prefer among the negotiated ones
        audiocodec: VideoRoomAudioCodec,
        /// video codec to prefer among the negotiated ones
        videocodec: VideoRoomVideoCodec,
        /// bitrate cap to return via REMB, overrides the global room value if present
        bitrate: u64,
        /// whether this publisher should be recorded or not
        record: bool,
        /// if recording, the base path/file to use for the recording files
        filename: String,
        /// display name to use in the room
        display: String,
        /// if provided, overrided the room audio_level_average for this user
        audio_level_average: u64,
        /// if provided, overrided the room audio_active_packets for this user
        audio_active_packets: u64,
        /// descriptions (names) for the published streams
        descriptions: Vec<VideoRoomPublishDescriptionParams>,
        secret: String,
        /// To force a renegotiation and/or an ICE restart
        update: bool,
        /// To force a renegotiation and/or an ICE restart
        restart: bool,
    }
);

make_dto!(
    VideoRoomPublishDescriptionParams,
    required {
        /// unique mid of a stream being published
        mid: String,
        /// text description of the stream (e.g., My front webcam)
        description: String
    }
);

make_dto!(
    VideoRoomSubscriberJoinParams,
    required { room: JanusId },
    optional {
        /// whether subscriptions should include a msid that references the publisher; false by default
        use_msid: bool,
        /// whether a new SDP offer is sent automatically when a subscribed publisher leaves; true by default
        autoupdate: bool,
        /// unique ID of the publisher that originated this request;
        /// optional, unless mandated by the room configuration
        private_id: u32, /* From janus-gateway source code, this id is never a string so doesn't have to be JanusId */
        /// list of media streams to subscribe to
        streams: Vec<VideoRoomSubscriberJoinStream>
    }
);

make_dto!(
    VideoRoomSubscriberJoinStream,
    required {
        /// unique ID of publisher owning the stream to subscribe to
        feed: JanusId
    },
    optional {
        /// unique mid of the publisher stream to subscribe to
        mid: String,
        /// id to map this subscription with entries in streams list
        crossrefid: u64,

        send: bool,
        substream: u64,
        temporal: u64,
        spatial_layer: u64,
        temporal_layer: u64,
        min_delay: u64,
        max_delay: u64,
    }
);

make_dto!(
    VideoRoomSubscriberUnsubscribeStream,
    optional {
        /// unique ID of publisher owning the stream to subscribe to
        feed: JanusId,
        /// unique mid of the publisher stream to subscribe to
        mid: String,
        /// id to map this subscription with entries in streams list
        sub_mid: u64
    }
);

make_dto!(
    VideoRoomEnableRecordingParams,
    required {
        room: JanusId,
        /// whether participants in this room should be automatically recorded or not
        record: bool,
    },
    optional {
        /// room secret, mandatory if configured
        secret: String
    }
);

make_dto!(
    VideoRoomListForwardersParams,
    required { room: JanusId },
    optional { secret: String }
);

make_dto!(
    VideoRoomStopRtpForward,
    required {
        room: JanusId,
        publisher_id: JanusId,
        stream_id: u64,
    }
);

make_dto!(VideoRoomListParticipantsParams, required { room: JanusId });

make_dto!(VideoRoomExistsParams, required { room: JanusId });

make_dto!(VideoRoomSubscribeParams, required { streams: Vec<VideoRoomSubscriberJoinStream> });

make_dto!(VideoRoomUnsubscribeParams, required { streams: Vec<VideoRoomSubscriberUnsubscribeStream> });

make_dto!(
    VideoRoomCombinedUpdateParams,
    required {
        subscribe: Vec<VideoRoomSubscriberJoinStream>,
        unsubscribe: Vec<VideoRoomSubscriberUnsubscribeStream>
    }
);

make_dto!(VideoRoomSwitchParams, required { streams: Vec<VideoRoomSwitchStream> });

make_dto!(
    VideoRoomPublisherConfigureParams,
    optional {
        audio: bool,
        video: bool,
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
        /// list of streams to configure
        streams: Vec<VideoRoomConfigurePublisherStream>,
        /// descriptions (names) for the published streams
        descriptions: Vec<VideoRoomPublishDescriptionParams>,
        /// video codec to prefer among the negotiated ones
        videocodec: VideoRoomVideoCodec
    }
);

make_dto!(
    VideoRoomConfigurePublisherStream,
    required { mid: String },
    optional {
        /// whether we should send this publisher a keyframe request
        keyframe: bool,
        /// depending on whether the media addressed by the above mid should be relayed or not
        send: bool,
        /// minimum delay to enforce via the playout-delay RTP extension, in blocks of 10ms
        min_delay: u64,
        /// maximum delay to enforce via the playout-delay RTP extension, in blocks of 10ms
        max_delay: u64
    }
);

make_dto!(
    VideoRoomPublisherJoinAndConfigureParams,
    required {
        #[serde(flatten)]
        join_params: VideoRoomPublisherJoinParams,
        #[serde(flatten)]
        configure_params: VideoRoomPublisherConfigureParams
    }
);

make_dto!(
    VideoRoomConfigureSubscriberParams,
    required {
        /// list of streams to configure
        streams: Vec<VideoRoomConfigureSubscriberStream>
    },
    optional {
        /// trigger an ICE restart
        restart: bool
    }
);

make_dto!(
    VideoRoomConfigureSubscriberStream,
    required {
        /// mid of the m-line to refer to
        mid: String
    },
    optional {
        /// depending on whether the mindex media should be relayed or not
        send: bool,
        /// substream to receive (0-2), in case simulcasting is enabled
        substream: u64,
        /// temporal layers to receive (0-2), in case simulcasting is enabled
        temporal: u64,
        /// How much time (in us, default 250000) without receiving packets will make us drop to the substream below
        fallback: u64,
        /// spatial layer to receive (0-2), in case SVC is enabled
        spacial_layer: u64,
        /// temporal layers to receive (0-2), in case SVC is enabled
        temporal_layer: u64,
        /// if provided, overrides the room `audio_level_average` for this user
        audio_level_average: u64,
        /// if provided, overrides the room `audio_active_packets` for this user
        audio_active_packets: u64,
        /// minimum delay to enforce via the playout-delay RTP extension, in blocks of 10ms
        min_delay: u64,
        /// maximum delay to enforce via the playout-delay RTP extension, in blocks of 10ms
        max_delay: u64
    }
);

make_dto!(
    VideoRoomSwitchStream,
    required {
        /// unique ID of the publisher the new source is from
        feed: JanusId,
        /// unique mid of the source we want to switch to
        mid: String,
        /// unique mid of the stream we want to pipe the new source to
        sub_mid: String
    },
    optional {
        substream: u64,
        temporal: u64,
        spatial_layer: u64,
        temporal_layer: u64,
    }
);

make_dto!(
    VideoRoomRtpForwardParams,
    required {
        room: JanusId,
        /// unique numeric ID of the publisher to relay externally
        publisher_id: JanusId,
        /// host address to forward the RTP and data packets to
        host: String,
        /// ipv4|ipv6, if we need to resolve the host address to an IP; by default, whatever we get
        #[serde(skip_serializing_if = "Vec::is_empty")]
        streams: Vec<VideoRoomRtpForwardStreamParams>
    },
    optional {
        secret: String,
        /// If `lock_rtp_forward` is set in the plugin settings, the `admin_key` (also configured in plugin settings) has to be supplied with RTP forwarding requests
        admin_key: String,
        /// length of authentication tag (32 or 80)
        host_family: String,
        /// length of authentication tag (32 or 80)
        srtp_suite: u16,
        /// key to use as crypto (base64 encoded key as in SDES)
        srtp_crypto: String
    }
);

make_dto!(
    VideoRoomRtpForwardStreamParams,
    required {
        /// mid of publisher stream to forward
        mid: String,
        /// port to forward the packets to
        port: u16
    },
    optional {
        secret: String,
        /// host address to forward the packets to, will use global one if missing
        host: String,
        host_family: String,
        /// port to contact to receive RTCP feedback from the recipient, and only for RTP streams, not data
        rtcp_port: String,
        /// SSRC to use when forwarding, and only for RTP streams, not data
        ssrc: String,
        /// payload type to use when forwarding, and only for RTP streams, not data
        pt: String,
        /// set to true if the source is simulcast and you want the forwarder to act as a regular viewer
        /// (single stream being forwarded) or false otherwise (substreams forwarded separately), default=false
        simulcast: bool,
        srtp_suite: u16,
        srtp_crypto: String,
        /// if video and simulcasting, port to forward the packets from the second substream/layer to
        port_2: u16,
        /// if video and simulcasting, SSRC to use the second substream/layer
        ssrc_2: String,
        /// if video and simulcasting, payload type to use the second substream/layer
        pt_2: String,
        /// if video and simulcasting, port to forward the packets from the third substream/layer to
        port_3: u16,
        /// if video and simulcasting, SSRC to use the third substream/layer
        ssrc_3: String,
        /// if video and simulcasting, payload type to use the third substream/layer
        pt_3: String
    }
);
