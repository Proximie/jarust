use jarust::jaconfig::JaConfig;
use jarust::jaconfig::TransportType;
use jarust::japlugin::Attach;
use serde_json::json;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut connection = jarust::connect(
        JaConfig::new("ws://localhost:8188/ws", None, "janus"),
        TransportType::Ws,
    )
    .await?;
    let session = connection.create(10).await?;
    let (handle, mut event_receiver) = session.attach("janus.plugin.echotest").await?;

    handle
        .message(json!({
            "video": true,
            "audio": true,
        }))
        .await?;

    while let Some(event) = event_receiver.recv().await {
        tracing::info!("response: {event:?}");
    }

    Ok(())
}
