use jarust::core::connect;
use jarust::core::jaconfig::JaConfig;
use jarust::core::jaconfig::JanusAPI;
use jarust::interface::tgenerator::RandomTransactionGenerator;
use jarust::plugins::streaming::jahandle_ext::Streaming;
use jarust::plugins::streaming::params::*;
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
    let (handle, mut events) = session.attach_streaming(timeout).await?;

    tokio::spawn(async move {
        while let Some(e) = events.recv().await {
            tracing::info!("{e:#?}");
        }
    });

    let mountpoint_id = handle
        .create_mountpoint(
            StreamingCreateParams {
                mountpoint_type: StreamingMountpointType::RTP,
                optional: StreamingCreateParamsOptional {
                    id: Some(JanusId::Uint(1337.try_into().unwrap())),
                    name: Some(String::from("stream name")),
                    description: Some(String::from("stream description")),
                    media: Some(vec![StreamingRtpMedia {
                        required: StreamingRtpMediaRequired {
                            media_type: StreamingRtpMediaType::VIDEO,
                            mid: String::from("v"),
                            port: 0,
                        },
                        optional: StreamingRtpMediaOptional {
                            pt: Some(100),
                            codec: Some(String::from("vp8")),
                            ..Default::default()
                        },
                    }]),
                    ..Default::default()
                },
            },
            timeout,
        )
        .await?
        .stream
        .id;

    let mountpoints = handle.list(timeout).await?;
    tracing::info!("Mountpoints {:#?}", mountpoints);

    let info = handle
        .info(JanusId::Uint(1337.try_into().unwrap()), None, timeout)
        .await?;
    tracing::info!("Info: {:#?}", info);

    handle
        .destroy_mountpoint(
            StreamingDestroyParams {
                id: mountpoint_id,
                optional: Default::default(),
            },
            timeout,
        )
        .await?;

    Ok(())
}
