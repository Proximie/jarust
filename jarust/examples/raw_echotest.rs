use jarust::core::connect;
use jarust::core::jaconfig::JaConfig;
use jarust::core::jaconfig::JanusAPI;
use jarust::core::prelude::Attach;
use serde_json::json;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("jarust_core=trace".parse()?))
        .init();
    let config = JaConfig {
        url: "ws://localhost:8188/ws".to_string(),
        apisecret: None,
        server_root: "janus".to_string(),
        capacity: 32,
    };
    let tgenerator = || uuid::Uuid::new_v4().to_string();
    let mut connection = connect(config, JanusAPI::WebSocket, tgenerator).await?;
    let timeout = Duration::from_secs(10);
    let session = connection
        .create_session(10, Duration::from_secs(10))
        .await?;
    let (handle, mut event_receiver) = session
        .attach("janus.plugin.echotest".to_string(), timeout)
        .await?;

    handle
        .send_waiton_ack(
            json!({
                "video": true,
                "audio": true,
            }),
            Duration::from_secs(2),
        )
        .await?;

    while let Some(event) = event_receiver.recv().await {
        tracing::info!("response: {event:#?}");
    }

    Ok(())
}
