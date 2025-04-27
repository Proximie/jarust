#![allow(unused_labels)]

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
#[case::multistream_ws("ws://localhost:8188/ws", JanusAPI::WebSocket)]
#[case::multistream_restful("http://localhost:8088", JanusAPI::Restful)]
#[case::legacy_ws("ws://localhost:9188/ws", JanusAPI::WebSocket)]
#[case::legacy_restful("http://localhost:9088", JanusAPI::Restful)]
#[tokio::test]
async fn core_test(#[case] url: &str, #[case] api: JanusAPI) {
    let config = JaConfig {
        url: url.to_string(),
        apisecret: None,
        server_root: "janus".to_string(),
        capacity: 32,
    };
    let mut connection = connect(config, api, RandomTransactionGenerator)
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
            "Server name should match the one in server_config/janus.cfg"
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
