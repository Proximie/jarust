#![allow(unused_labels)]

use e2e::TestingEnv;
use jarust::core::jaconfig::JaConfig;
use jarust::core::jaconfig::JanusAPI;
use jarust::interface::tgenerator::RandomTransactionGenerator;
use jarust::plugins::common::U63;
use jarust::plugins::legacy_video_room::events::PluginEvent;
use jarust::plugins::legacy_video_room::handle::LegacyVideoRoomHandle;
use jarust::plugins::legacy_video_room::jahandle_ext::LegacyVideoRoom;
use jarust::plugins::legacy_video_room::params::LegacyVideoRoomCreateParams;
use jarust::plugins::legacy_video_room::params::LegacyVideoRoomExistsParams;
use jarust::plugins::JanusId;
use rand::{thread_rng, Rng};
use rstest::*;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;

#[rstest]
#[case::legacy_ws(TestingEnv::Legacy(JanusAPI::WebSocket))]
#[case::legacy_restful(TestingEnv::Legacy(JanusAPI::Restful))]
#[tokio::test]
async fn legacy_videoroom_room_crud_e2e(#[case] testing_env: TestingEnv) {
    let default_timeout = Duration::from_secs(4);
    let handle = make_legacy_videoroom_attachment(testing_env).await.0;
    let mut rng = thread_rng();
    let room_id = JanusId::Uint(rng.gen_range(0..U63::MAX).try_into().unwrap());

    'before_creation: {
        let exists = handle
            .exists(
                LegacyVideoRoomExistsParams {
                    room: room_id.clone(),
                },
                default_timeout,
            )
            .await
            .expect("Failed to check if room exists; before_creation");
        assert!(!exists, "Room should not exist before creation");
    }

    'creation: {
        handle
            .create_room(
                LegacyVideoRoomCreateParams {
                    room: Some(room_id.clone()),
                    ..Default::default()
                },
                default_timeout,
            )
            .await
            .expect("Failed to create room; creation");

        let exists = handle
            .exists(
                LegacyVideoRoomExistsParams {
                    room: room_id.clone(),
                },
                default_timeout,
            )
            .await
            .expect("Failed to check if room exists; creation");
        assert!(exists, "Room should exist after creation");
    }
}

async fn make_legacy_videoroom_attachment(
    testing_env: TestingEnv,
) -> (LegacyVideoRoomHandle, UnboundedReceiver<PluginEvent>) {
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
        .attach_legacy_video_room(timeout)
        .await
        .expect("Failed to attach plugin");

    (handle, event_receiver)
}
