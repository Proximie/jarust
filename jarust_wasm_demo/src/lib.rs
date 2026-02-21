use jarust_core::connect;
use jarust_core::jaconfig::JaConfig;
use jarust_core::jaconfig::JanusAPI;
use jarust_interface::tgenerator::UuidTransactionGenerator;
use jarust_plugins::echo_test::jahandle_ext::EchoTest;
use std::time::Duration;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn run() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    if let Err(e) = run_inner().await {
        tracing::error!("Fatal error: {:?}", e);
    }
}

async fn run_inner() -> Result<(), jarust_interface::Error> {
    let config = JaConfig {
        url: "wss://janus.conf.meetecho.com/ws".to_string(),
        apisecret: None,
        server_root: "janus".to_string(),
        capacity: 32,
    };

    tracing::info!("Connecting to Janus...");
    let mut connection = connect(config, JanusAPI::WebSocket, UuidTransactionGenerator).await?;
    tracing::info!("Connected");

    let timeout = Duration::from_secs(10);
    let session = connection.create_session(10, timeout).await?;
    tracing::info!("Session created");

    let (handle, mut event_receiver) = session.attach_echo_test(timeout).await?;
    tracing::info!("Attached to echo test plugin");

    // Send a test message
    handle
        .fire_and_forget(serde_json::json!({
            "audio": true,
            "video": true,
        }))
        .await?;
    tracing::info!("Sent fire-and-forget message");

    handle
        .send_waiton_ack(
            serde_json::json!({
                "audio": true,
                "video": true,
            }),
            timeout,
        )
        .await?;
    tracing::info!("Got ack");

    // Drain a few events
    for _ in 0..3 {
        if let Some(event) = event_receiver.recv().await {
            tracing::info!("Event: {event:#?}");
        }
    }

    Ok(())
}
