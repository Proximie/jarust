use jarust::jaconfig::JaConfig;
use jarust::jaconfig::TransportType;
use jarust::TransactionGenerationStrategy;
use jarust_plugins::audio_bridge::jahandle_ext::AudioBridge;
use jarust_plugins::audio_bridge::msg_opitons::CreateRoomOptions;
use jarust_plugins::audio_bridge::msg_opitons::JoinRoomOptions;
use std::path::Path;
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let filename = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    let env_filter = EnvFilter::from_default_env()
        .add_directive("jarust=trace".parse()?)
        .add_directive(format!("{filename}=trace").parse()?);
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
    let timeout = std::time::Duration::from_secs(10);

    let config = JaConfig::builder()
        .url("ws://localhost:8188/ws")
        .capacity(32)
        .build();
    let mut connection = jarust::connect(
        config,
        TransportType::Ws,
        TransactionGenerationStrategy::Random,
    )
    .await?;
    let session = connection.create(10, timeout).await?;
    let (handle, mut event_receiver) = session.attach_audio_bridge().await?;

    let create_room_rsp = handle
        .create_room_with_config(
            CreateRoomOptions {
                secret: Some("superdupersecret".to_string()),
                ..Default::default()
            },
            timeout,
        )
        .await?;

    tracing::info!(
        "Created Room {:#?}, permanent: {}",
        create_room_rsp.room,
        create_room_rsp.permanent
    );

    handle
        .join_room(
            create_room_rsp.room.clone(),
            JoinRoomOptions {
                secret: Some("superdupersecret".to_string()),
                generate_offer: Some(true),
                ..Default::default()
            },
            None,
            timeout,
        )
        .await?;

    if let Some(event) = event_receiver.recv().await {
        tracing::info!("Joined Room {:#?}, {:#?}", create_room_rsp.room, event);
    }

    let list_participants_rsp = handle
        .list_participants(create_room_rsp.room, timeout)
        .await?;
    tracing::info!(
        "Participants in room {:#?}: {:#?}",
        list_participants_rsp.room,
        list_participants_rsp.participants
    );

    handle.leave(timeout).await?;

    if let Some(event) = event_receiver.recv().await {
        tracing::info!("Event {:#?}", event);
    }

    Ok(())
}
