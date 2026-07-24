use jarust::core::custom_connect;
use jarust::core::prelude::Attach;
use jarust::interface::janus_interface::ConnectionParams;
use jarust::interface::janus_interface::JanusInterface;
use jarust::interface::socketio::SocketIoInterface;
use jarust::interface::tgenerator::RandomTransactionGenerator;
use serde_json::json;
use std::path::Path;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let filename = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    let env_filter = EnvFilter::from_default_env()
        .add_directive("jarust_core=trace".parse()?)
        .add_directive("jarust_interface=trace".parse()?)
        .add_directive(format!("{filename}=trace").parse()?);
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let conn_params = ConnectionParams {
        url: "".to_string(),
        capacity: 32,
        apisecret: Some("".to_string()),
        server_root: "janus".to_string(),
    };

    let interface =
        SocketIoInterface::make_interface(conn_params, RandomTransactionGenerator).await?;

    let mut connection = custom_connect(interface).await?;
    let timeout = Duration::from_secs(10);

    tracing::info!("server info: {:#?}", connection.server_info(timeout).await?);

    let session = connection.create_session(10, timeout).await?;
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
