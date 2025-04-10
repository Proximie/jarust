use jarust::core::connect;
use jarust::core::jaconfig::JaConfig;
use jarust::core::jaconfig::JanusAPI;
use jarust::core::japlugin::Attach;
use jarust::interface::japrotocol::Jsep;
use jarust::interface::japrotocol::JsepType;
use jarust::interface::tgenerator::RandomTransactionGenerator;
use serde_json::json;
use std::path::Path;
use std::time::Duration;
use tokio::time;
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let filename = Path::new(file!()).file_stem().unwrap().to_str().unwrap();
    let env_filter = EnvFilter::from_default_env()
        .add_directive("jarust_core=trace".parse()?)
        .add_directive(format!("{filename}=trace").parse()?);
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let config = JaConfig {
        url: "https://janus.conf.meetecho.com".to_string(),
        apisecret: None,
        server_root: "janus".to_string(),
        capacity: 32,
    };
    let mut connection = connect(config, JanusAPI::Restful, RandomTransactionGenerator).await?;
    let timeout = Duration::from_secs(10);

    let session = connection
        .create_session(10, Duration::from_secs(10))
        .await?;
    tracing::info!("server info: {:#?}", connection.server_info(timeout).await?);
    let (handle, mut event_receiver) = session
        .attach("janus.plugin.echotest".to_string(), timeout)
        .await?;

    tokio::spawn(async move {
        let mut interval = time::interval(time::Duration::from_secs(2));

        loop {
            handle
                .send_waiton_ack(
                    json!({
                        "video": true,
                        "audio": true,
                    }),
                    Duration::from_secs(10),
                )
                .await
                .unwrap();

            handle
                .fire_and_forget_with_jsep(
                    json!({
                        "video": true,
                        "audio": true,
                    }),
                    Jsep {
                        sdp: "".to_string(),
                        trickle: Some(false),
                        jsep_type: JsepType::Offer,
                    },
                )
                .await
                .unwrap();

            interval.tick().await;
        }
    });

    while let Some(event) = event_receiver.recv().await {
        tracing::info!("response: {event:#?}");
    }

    Ok(())
}
