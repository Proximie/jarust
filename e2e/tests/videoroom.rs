use jarust::core::jaconfig::JaConfig;
use jarust::core::jaconfig::JanusAPI;
use jarust::interface::tgenerator::RandomTransactionGenerator;
use jarust::plugins::video_room::events::PluginEvent;
use jarust::plugins::video_room::events::VideoRoomEvent;
use jarust::plugins::video_room::handle::VideoRoomHandle;
use jarust::plugins::video_room::jahandle_ext::VideoRoom;
use jarust::plugins::video_room::params::VideoRoomDestroyParams;
use jarust::plugins::video_room::params::VideoRoomEditParams;
use jarust::plugins::video_room::params::VideoRoomEditParamsOptional;
use jarust::plugins::video_room::params::VideoRoomExistsParams;
use jarust::plugins::video_room::params::VideoRoomPublisherJoinParams;
use jarust::plugins::video_room::params::VideoRoomPublisherJoinParamsOptional;
use jarust::plugins::video_room::responses::VideoRoomParticipant;
use jarust::plugins::JanusId;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;

#[allow(unused_labels)]
#[tokio::test]
async fn room_crud_e2e() {
    let default_timeout = Duration::from_secs(4);
    let handle = make_videoroom_attachment().await.0;
    let room_id = JanusId::Uint(rand::random::<u64>().into());

    'before_creation: {
        let exists = handle
            .exists(
                VideoRoomExistsParams {
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
            .create_room(Some(room_id.clone()), default_timeout)
            .await
            .expect("Failed to create room; creation");
        let exists = handle
            .exists(
                VideoRoomExistsParams {
                    room: room_id.clone(),
                },
                default_timeout,
            )
            .await
            .expect("Failed to check if room exists; creation");
        assert!(exists, "Room should exist after creation");

        let rooms = handle
            .list_rooms(default_timeout)
            .await
            .expect("Failed to list rooms; creation");
        assert!(
            rooms.iter().any(|room| room.room == room_id),
            "Room should be visible when listing rooms"
        );
    }

    'description_edit: {
        handle
            .edit_room(
                VideoRoomEditParams {
                    room: room_id.clone(),
                    optional: VideoRoomEditParamsOptional {
                        new_description: Some("new description".to_string()),
                        ..Default::default()
                    },
                },
                default_timeout,
            )
            .await
            .expect("Failed to edit room; description_edit");

        let rooms = handle
            .list_rooms(default_timeout)
            .await
            .expect("Failed to list rooms; description_edit");
        let edit_room = rooms
            .iter()
            .find(|room| room.room == room_id)
            .expect("Room not found; description_edit");
        assert_eq!(
            edit_room.description,
            "new description".to_string(),
            "Room description should be updated"
        );
    }

    'private_edit: {
        handle
            .edit_room(
                VideoRoomEditParams {
                    room: room_id.clone(),
                    optional: VideoRoomEditParamsOptional {
                        new_is_private: Some(true),
                        ..Default::default()
                    },
                },
                default_timeout,
            )
            .await
            .expect("Failed to edit room; private_edit");

        let rooms = handle
            .list_rooms(default_timeout)
            .await
            .expect("Failed to list rooms; private_edit");
        assert!(
            !rooms.iter().any(|room| room.room == room_id),
            "Room should not be visible when listing rooms"
        );
        let exists = handle
            .exists(
                VideoRoomExistsParams {
                    room: room_id.clone(),
                },
                default_timeout,
            )
            .await
            .expect("Failed to check if room exists; private_edit");
        assert!(exists, "Room should exist after setting to private");
    }

    'destroy: {
        handle
            .destroy_room(
                VideoRoomDestroyParams {
                    room: room_id.clone(),
                    optional: Default::default(),
                },
                default_timeout,
            )
            .await
            .expect("Failed to destroy room; destroy");
        let exists = handle
            .exists(
                VideoRoomExistsParams {
                    room: room_id.clone(),
                },
                default_timeout,
            )
            .await
            .expect("Failed to check if room exists; destroy");
        assert!(!exists, "Room should not exist after destruction");
    }
}

#[allow(unused_labels)]
#[tokio::test]
async fn participants_e2e() {
    let default_timeout = Duration::from_secs(4);
    let room_id = JanusId::Uint(rand::random::<u64>().into());
    let admin = make_videoroom_attachment().await.0;
    let (alice_handle, mut alice_events) = make_videoroom_attachment().await;
    let (bob_handle, mut bob_events) = make_videoroom_attachment().await;
    let (eve_handle, mut eve_events) = make_videoroom_attachment().await;

    admin
        .create_room(Some(room_id.clone()), default_timeout)
        .await
        .expect("Admin failed to create room; creation");

    // Alice joins the room
    let alice = {
        let display = Some("Alice".to_string());
        alice_handle
            .join_as_publisher(
                VideoRoomPublisherJoinParams {
                    room: room_id.clone(),
                    optional: VideoRoomPublisherJoinParamsOptional {
                        display: display.clone(),
                        ..Default::default()
                    },
                },
                None,
                default_timeout,
            )
            .await
            .expect("Alice failed to join room");

        let PluginEvent::VideoRoomEvent(VideoRoomEvent::PublisherJoined {
            id,
            room,
            publishers,
            ..
        }) = alice_events
            .recv()
            .await
            .expect("Alice failed to receive event")
        else {
            panic!("Alice received unexpected event")
        };

        assert_eq!(room, room_id, "Alice should join correct room");
        assert_eq!(publishers, vec![], "No publishers should be in room");

        VideoRoomParticipant {
            id,
            display,
            publisher: false,
            talking: Some(false),
        }
    };
}

async fn make_videoroom_attachment() -> (VideoRoomHandle, UnboundedReceiver<PluginEvent>) {
    let config = JaConfig {
        url: "ws://localhost:8188/ws".to_string(),
        apisecret: None,
        server_root: "janus".to_string(),
        capacity: 32,
    };
    let mut connection =
        jarust::core::connect(config, JanusAPI::WebSocket, RandomTransactionGenerator)
            .await
            .expect("Failed to connect to server");
    let timeout = Duration::from_secs(10);
    let session = connection
        .create_session(10, Duration::from_secs(10))
        .await
        .expect("Failed to create session");
    let (handle, event_receiver) = session
        .attach_video_room(timeout)
        .await
        .expect("Failed to attach plugin");

    (handle, event_receiver)
}
