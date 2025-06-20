use e2e::TestingEnv;
use jarust::core::jaconfig::JaConfig;
use jarust::core::jaconfig::JanusAPI;
use jarust::interface::Error;
use jarust::interface::tgenerator::RandomTransactionGenerator;
use jarust::plugins::JanusId;
use jarust::plugins::common::U63;
use jarust::plugins::streaming::events::PluginEvent;
use jarust::plugins::streaming::handle::StreamingHandle;
use jarust::plugins::streaming::params::*;
use jarust::plugins::streaming::jahandle_ext::Streaming;
use rand::{thread_rng, Rng};
use rstest::*;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;

#[rstest]
#[case::multistream_ws(TestingEnv::Multistream(JanusAPI::WebSocket))]
#[case::multistream_restful(TestingEnv::Multistream(JanusAPI::Restful))]
#[tokio::test]
async fn streaming_crud_e2e(#[case] testing_env: TestingEnv) {
    let default_timeout = Duration::from_secs(4);
    let handle = make_streaming_attachment(testing_env).await.0;
    let mut rng = thread_rng();
    let stream_id = JanusId::Uint(rng.gen_range(0..U63::MAX).try_into().unwrap());

    'before_creation: {
        let info_err = handle
            .info(stream_id.clone(), None, default_timeout)
        .await
        .expect_err("Stream should not exist before creation; before_creation");
        let Error::PluginResponseError{ error_code, error } = info_err else {
            panic!("Unexpected non PluginResponseError");
        };
        assert_eq!(error_code, 455); // JANUS_ERROR_INVALID_JSON_OBJECT
        assert!(error.starts_with("No such mountpoint/stream"));
    }

    'creation: {
        let mp_id = handle
            .create_mountpoint(
                StreamingCreateParams {
                    mountpoint_type: StreamingMountpointType::RTP,
                    optional: StreamingCreateParamsOptional {
                        id: Some(stream_id.clone()),
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
                default_timeout,
            )
            .await
            .expect("Failed to create mountpoint; creation")
            .stream
            .id;
        assert_eq!(mp_id, stream_id.clone());

        let info = handle
            .info(stream_id.clone(), None, default_timeout)
        .await
        .expect("Failed to check if mountpoint exists; creation");
        assert_eq!(info.id, stream_id.clone());

        let mountpoints = handle
            .list(default_timeout)
            .await
            .expect("Failed to list mountpoints; creation");
        assert!(
            mountpoints.iter().any(|mp| mp.id == stream_id.clone()),
            "Mountpoint should be visible when listing mountpoints"
        );
    }

    'destroy: {
        handle
            .destroy_mountpoint(
                StreamingDestroyParams {
                    id: stream_id.clone(),
                    optional: Default::default(),
                },
                default_timeout,
            )
            .await
            .expect("Failed to destroy mountpoint; destroy");
        let info_err = handle
            .info(stream_id.clone(), None, default_timeout)
        .await
        .expect_err("Stream should not exist after destruction; destroy");
        let Error::PluginResponseError{ error_code, error } = info_err else {
            panic!("Unexpected non PluginResponseError");
        };
        assert_eq!(error_code, 455); // JANUS_ERROR_INVALID_JSON_OBJECT
        assert!(error.starts_with("No such mountpoint/stream"));
    }
}

async fn make_streaming_attachment(
    testing_env: TestingEnv,
) -> (StreamingHandle, UnboundedReceiver<PluginEvent>) {
    let config = JaConfig {
        url: testing_env.url().to_string(),
        apisecret: None,
        server_root: "janus".to_string(),
        capacity: 32,
    };
    let mut connection =
        jarust::core::connect(config, testing_env.api(), RandomTransactionGenerator)
            .await
            .expect("Failed to connect to server");
    let timeout = Duration::from_secs(10);
    let session = connection
        .create_session(10, Duration::from_secs(10))
        .await
        .expect("Failed to create session");
    let (handle, event_receiver) = session
        .attach_streaming(timeout)
        .await
        .expect("Failed to attach plugin");

    (handle, event_receiver)
}
