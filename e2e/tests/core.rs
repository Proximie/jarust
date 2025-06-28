#![allow(unused_labels)]

use e2e::TestingEnv;
use jarust::core::connect;
use jarust::core::jaconfig::JaConfig;
use jarust::core::jaconfig::JanusAPI;
use jarust::core::prelude::Attach;
use jarust::interface::error::Error::JanusError;
use jarust::interface::japrotocol::GenericEvent;
use jarust::interface::japrotocol::JaHandleEvent;
use jarust::interface::japrotocol::ResponseType;
use jarust::interface::tgenerator::RandomTransactionGenerator;
use rstest::*;
use std::time::Duration;

#[rstest]
#[case::multistream_ws(TestingEnv::Multistream(JanusAPI::WebSocket))]
#[case::multistream_restful(TestingEnv::Multistream(JanusAPI::Restful))]
#[case::legacy_ws(TestingEnv::Legacy(JanusAPI::WebSocket))]
#[case::legacy_restful(TestingEnv::Legacy(JanusAPI::Restful))]
#[tokio::test]
async fn core_test(#[case] testing_env: TestingEnv) {
    let config = JaConfig {
        url: testing_env.url().to_string(),
        apisecret: None,
        server_root: "janus".to_string(),
        capacity: 32,
    };
    let mut connection = connect(config, testing_env.api(), RandomTransactionGenerator)
        .await
        .unwrap();

    'server_info: {
        let info = connection
            .server_info(Duration::from_secs(5))
            .await
            .unwrap();
        assert_eq!(
            info.server_name,
            "Jarust".to_string(),
            "Server name should match the one in server_config/janus.jcfg"
        );
    }

    'destroyed_session: {
        let session = connection
            .create_session(10, Duration::from_secs(5))
            .await
            .unwrap();

        session.destroy(Duration::from_secs(5)).await.unwrap();

        let result = session
            .attach("janus.plugin.echotest".to_string(), Duration::from_secs(5))
            .await;
        assert!(
            matches!(result, Err(JanusError { code: _, reason: _ })),
            "No such session after destroying it"
        )
    }

    let session = connection
        .create_session(10, Duration::from_secs(5))
        .await
        .unwrap();

    let (handle, mut event_recv) = session
        .attach("janus.plugin.echotest".to_string(), Duration::from_secs(5))
        .await
        .unwrap();

    handle.detach().await.unwrap();
    assert_eq!(
        event_recv.recv().await.unwrap().janus,
        ResponseType::Event(JaHandleEvent::GenericEvent(GenericEvent::Detached)),
        "Hangup event should be received"
    );
}
