use jarust::core::connect;
use jarust::core::jaconfig::JaConfig;
use jarust::core::jaconfig::JanusAPI;
use jarust::interface::japrotocol::Jsep;
use jarust::interface::japrotocol::JsepType;
use jarust::interface::tgenerator::RandomTransactionGenerator;
use jarust::plugins::video_room::jahandle_ext::VideoRoom;
use jarust::plugins::video_room::params::*;
use jarust::plugins::JanusId;
use std::path::Path;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let filename = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    let env_filter = EnvFilter::from_default_env()
        .add_directive("jarust_core=trace".parse()?)
        .add_directive("jarust_plugins=trace".parse()?)
        .add_directive("jarust_interface=trace".parse()?)
        .add_directive("jarust_rt=trace".parse()?)
        .add_directive(format!("{filename}=trace").parse()?);
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let timeout = Duration::from_secs(10);
    let config = JaConfig {
        url: "ws://localhost:8188/ws".to_string(),
        apisecret: None,
        server_root: "janus".to_string(),
        capacity: 32,
    };
    let mut connection = connect(config, JanusAPI::WebSocket, RandomTransactionGenerator).await?;
    let session = connection
        .create_session(10, Duration::from_secs(10))
        .await?;
    let (handle, mut events) = session.attach_video_room(timeout).await?;

    tokio::spawn(async move {
        while let Some(e) = events.recv().await {
            tracing::info!("{e:#?}");
        }
    });

    let room_id = handle
        .create_room_with_config(
            VideoRoomCreateParams {
                audiocodec: Some(VideoRoomAudioCodecList::new(vec![
                    VideoRoomAudioCodec::OPUS,
                    VideoRoomAudioCodec::PCMU,
                ])),
                videocodec: Some(VideoRoomVideoCodecList::new(vec![VideoRoomVideoCodec::VP8])),
                notify_joining: Some(true),
                ..Default::default()
            },
            timeout,
        )
        .await?
        .room;

    handle
        .edit_room(
            VideoRoomEditParams {
                room: room_id.clone(),
                optional: Default::default(),
            },
            timeout,
        )
        .await?;

    let exists = handle
        .exists(
            VideoRoomExistsParams {
                room: room_id.clone(),
            },
            timeout,
        )
        .await?;
    tracing::info!(
        "Does the room we just created and edited exist? {:#?}",
        exists
    );

    let rooms = handle.list_rooms(timeout).await?;
    tracing::info!("Rooms {:#?}", rooms);

    let allowed_enable = handle
        .allowed(
            VideoRoomAllowedParams {
                room: room_id.clone(),
                action: VideoRoomAllowedAction::Enable,
                allowed: vec![],
                secret: None,
            },
            timeout,
        )
        .await?;
    tracing::info!("Allowed list: {:#?}", allowed_enable.allowed);
    let allowed_add = handle
        .allowed(
            VideoRoomAllowedParams {
                room: room_id.clone(),
                action: VideoRoomAllowedAction::Add,
                allowed: vec!["teststring".to_string(), "removeme".to_string()],
                secret: None,
            },
            timeout,
        )
        .await?;
    tracing::info!("Allowed list: {:#?}", allowed_add.allowed);
    let allowed_remove = handle
        .allowed(
            VideoRoomAllowedParams {
                room: room_id.clone(),
                action: VideoRoomAllowedAction::Remove,
                allowed: vec!["removeme".to_string()],
                secret: None,
            },
            timeout,
        )
        .await?;
    tracing::info!("Allowed list: {:#?}", allowed_remove.allowed);
    handle
        .allowed(
            VideoRoomAllowedParams {
                room: room_id.clone(),
                action: VideoRoomAllowedAction::Disable,
                allowed: vec![],
                secret: None,
            },
            timeout,
        )
        .await?;

    handle
        .join_as_publisher(
            VideoRoomPublisherJoinParams {
                room: room_id.clone(),
                optional: VideoRoomPublisherJoinParamsOptional {
                    id: Some(JanusId::Uint(1337.try_into().unwrap())),
                    display: Some(String::from("Publisher name")),
                    token: None,
                },
            },
            None,
            timeout,
        )
        .await?;

    handle
        .publish(
            VideoRoomPublishParams {
                audiocodec: Some(VideoRoomAudioCodec::OPUS),
                videocodec: Some(VideoRoomVideoCodec::H264),
                bitrate: Some(3500),
                descriptions: Some(vec![VideoRoomPublishDescriptionParams {
                    mid: String::from("stream-0"),
                    description: String::from("The ultimate stream!"),
                }]),
                ..Default::default()
            },
            Jsep {
                jsep_type: JsepType::Offer,
                trickle: Some(false),
                sdp: EXAMPLE_SDP_OFFER.to_string(),
            },
            timeout,
        )
        .await?;

    let list_participants_rsp = handle
        .list_participants(
            VideoRoomListParticipantsParams {
                room: room_id.clone(),
            },
            timeout,
        )
        .await?;
    tracing::info!(
        "Participants in room {:#?}: {:#?}",
        list_participants_rsp.room,
        list_participants_rsp.participants
    );

    handle.unpublish(timeout).await?;

    handle.leave(timeout).await?;

    let list_participants_rsp = handle
        .list_participants(
            VideoRoomListParticipantsParams {
                room: room_id.clone(),
            },
            timeout,
        )
        .await?;
    tracing::info!(
        "Participants in room {:#?}: {:#?}",
        list_participants_rsp.room,
        list_participants_rsp.participants
    );

    handle
        .destroy_room(
            VideoRoomDestroyParams {
                room: room_id,
                optional: Default::default(),
            },
            timeout,
        )
        .await?;

    Ok(())
}

const EXAMPLE_SDP_OFFER: &str = "v=0
o=rtc 2683980088 0 IN IP4 127.0.0.1
s=-
t=0 0
a=group:BUNDLE 0 1
a=group:LS 0 1
a=msid-semantic:WMS *
a=setup:actpass
a=ice-ufrag:eBRl
a=ice-pwd:+AWJI4q7V5ivTpOnEyzoHL
a=ice-options:ice2,trickle
a=fingerprint:sha-256 00:6B:85:04:41:D1:AF:31:18:C5:32:43:E9:0D:17:D9:31:8A:01:89:10:B8:9D:05:06:14:DA:97:F4:E1:74:81
m=audio 63582 UDP/TLS/RTP/SAVPF 111
c=IN IP4 172.20.10.6
a=mid:0
a=sendonly
a=ssrc:2724817378 cname:20fq0G5qdxVf2T7D
a=ssrc:2724817378 msid:zwaqhEaMoL3k0x9g zwaqhEaMoL3k0x9g-audio
a=msid:zwaqhEaMoL3k0x9g zwaqhEaMoL3k0x9g-audio
a=rtcp-mux
a=rtpmap:111 opus/48000/2
a=fmtp:111 minptime=10;maxaveragebitrate=96000;stereo=1;sprop-stereo=1;useinbandfec=1
a=candidate:2 1 UDP 2130706175 2a00:20:c341:539e:c18:aed5:7682:7662 63582 typ host
a=candidate:1 1 UDP 2122317823 172.20.10.6 63582 typ host
a=candidate:3 1 UDP 2122317311 192.168.39.104 63582 typ host
a=end-of-candidates
m=video 63582 UDP/TLS/RTP/SAVPF 96
c=IN IP4 172.20.10.6
a=mid:1
a=sendonly
a=ssrc:2724817379 cname:20fq0G5qdxVf2T7D
a=ssrc:2724817379 msid:zwaqhEaMoL3k0x9g zwaqhEaMoL3k0x9g-video
a=msid:zwaqhEaMoL3k0x9g zwaqhEaMoL3k0x9g-video
a=rtcp-mux
a=rtpmap:96 H264/90000
a=rtcp-fb:96 nack
a=rtcp-fb:96 nack pli
a=rtcp-fb:96 goog-remb
a=fmtp:96 profile-level-id=42e01f;packetization-mode=1;level-asymmetry-allowed=1";
