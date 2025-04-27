use e2e::TestingEnv;
use jarust::core::jaconfig::JaConfig;
use jarust::core::jaconfig::JanusAPI;
use jarust::interface::tgenerator::RandomTransactionGenerator;
use jarust::plugins::echo_test::events::EchoTestEvent;
use jarust::plugins::echo_test::events::PluginEvent;
use jarust::plugins::echo_test::jahandle_ext::EchoTest;
use jarust::plugins::echo_test::params::EchoTestStartParams;
use rstest::*;
use std::time::Duration;

#[rstest]
#[case::multistream_ws(TestingEnv::Multistream(JanusAPI::WebSocket))]
#[case::multistream_restful(TestingEnv::Multistream(JanusAPI::Restful))]
#[case::legacy_ws(TestingEnv::Legacy(JanusAPI::WebSocket))]
#[case::legacy_restful(TestingEnv::Legacy(JanusAPI::Restful))]
#[tokio::test]
async fn echotest_e2e(#[case] testing_env: TestingEnv) {
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
    let (handle, mut event_receiver) = session
        .attach_echo_test(timeout)
        .await
        .expect("Failed to attach plugin");

    handle
        .start(EchoTestStartParams {
            audio: Some(true),
            ..Default::default()
        })
        .await
        .expect("Failed to send start message");
    assert_eq!(
        event_receiver.recv().await,
        Some(PluginEvent::EchoTestEvent(EchoTestEvent::Result {
            echotest: "event".to_string(),
            result: "ok".to_string()
        }))
    );

    // Empty body should return an error
    handle
        .start(Default::default())
        .await
        .expect("Failed to send start message");
    assert!(matches!(
        event_receiver.recv().await,
        Some(PluginEvent::EchoTestEvent(EchoTestEvent::Error {
            error_code: _,
            error: _
        }))
    ));
}
