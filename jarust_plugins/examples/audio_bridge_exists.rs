use jarust::jaconfig::JaConfig;
use jarust::jaconfig::TransportType;
use jarust_plugins::audio_bridge::messages::AudioBridgeCreateOptions;
use jarust_plugins::audio_bridge::messages::AudioBridgeDestroyOptions;
use jarust_plugins::audio_bridge::AudioBridge;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut connection = jarust::connect(
        JaConfig::new("ws://localhost:8188/ws", None, "janus"),
        TransportType::Ws,
    )
    .await?;
    let session = connection.create(10).await?;
    let (handle, ..) = session.attach_audio_bridge().await?;

    let exist = handle.exists(4321).await?;
    tracing::info!("Room exists?: {}", exist);

    if !exist {
        let _ = handle
            .create_room_with_config(AudioBridgeCreateOptions {
                room: Some(4321),
                secret: Some("superdupersecret".to_string()),
                ..Default::default()
            })
            .await?;

        let exist = handle.exists(4321).await?;
        tracing::info!("Room exists?: {}", exist);

        if exist {
            let _ = handle
                .destroy_room(
                    4321,
                    AudioBridgeDestroyOptions {
                        secret: Some("superdupersecret".to_string()),
                        ..Default::default()
                    },
                )
                .await;

            let exist = handle.exists(4321).await?;
            tracing::info!("Room exists?: {}", exist);
        }
    }

    Ok(())
}
