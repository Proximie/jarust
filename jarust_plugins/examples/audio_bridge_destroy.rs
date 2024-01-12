use jarust::jaconfig::JaConfig;
use jarust::jaconfig::TransportType;
use jarust_plugins::audio_bridge::messages::AudioBridgeCreateOptions;
use jarust_plugins::audio_bridge::messages::AudioBridgeDestroyOptions;
use jarust_plugins::audio_bridge::AudioBridge;
use log::LevelFilter;
use log::SetLoggerError;
use simple_logger::SimpleLogger;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    init_logger()?;

    let mut connection = jarust::connect(JaConfig::new(
        "ws://localhost:8188/ws",
        None,
        TransportType::Wss,
        "janus",
    ))
    .await?;
    let session = connection.create(10).await?;
    let (handle, ..) = session.attach_audio_bridge().await?;

    let _ = handle
        .create_room_with_config(AudioBridgeCreateOptions {
            room: Some(4321),
            description: Some("A nice description".to_string()),
            secret: Some("superdupersecret".to_string()),
            ..Default::default()
        })
        .await?;

    let (room, permanent) = handle
        .destroy_room(
            4321,
            AudioBridgeDestroyOptions {
                secret: Some("superdupersecret".to_string()),
                ..Default::default()
            },
        )
        .await?;

    log::info!("Detroyed Room {}, permanent: {}", room, permanent);

    Ok(())
}

fn init_logger() -> Result<(), SetLoggerError> {
    SimpleLogger::new()
        .with_level(LevelFilter::Trace)
        .with_colors(true)
        .with_module_level("tokio_tungstenite", LevelFilter::Off)
        .with_module_level("tungstenite", LevelFilter::Off)
        .with_module_level("want", LevelFilter::Off)
        .init()
}
