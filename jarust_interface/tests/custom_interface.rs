//! End-to-end style test for [`CustomInterface`] driven by an in-memory loopback
//! [`Transport`]. The loopback parses each outbound Janus request, echoes back a canned
//! response carrying the same `transaction`, and thereby exercises the full protocol
//! path: transaction generation, request decoration, demultiplexing, routing and
//! response polling.

use bytes::Bytes;
use jarust_interface::custom_interface::CustomInterface;
use jarust_interface::janus_interface::ConnectionParams;
use jarust_interface::janus_interface::JanusInterface;
use jarust_interface::tgenerator::RandomTransactionGenerator;
use jarust_interface::transport_trait::Transport;
use serde_json::json;
use serde_json::Value;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::mpsc;

const TIMEOUT: Duration = Duration::from_secs(5);

/// A loopback transport: turns each outbound request into a canned response.
#[derive(Debug)]
struct LoopbackTransport {
    inbound: Mutex<Option<mpsc::UnboundedSender<Bytes>>>,
    session_id: u64,
    handle_id: u64,
}

impl LoopbackTransport {
    fn new(session_id: u64, handle_id: u64) -> Self {
        Self {
            inbound: Mutex::new(None),
            session_id,
            handle_id,
        }
    }

    fn reply(&self, response: Value) {
        if let Some(tx) = self.inbound.lock().unwrap().as_ref() {
            let _ = tx.send(Bytes::from(response.to_string()));
        }
    }
}

#[async_trait::async_trait]
impl Transport for LoopbackTransport {
    async fn connect(
        &self,
        _url: &str,
    ) -> Result<mpsc::UnboundedReceiver<Bytes>, jarust_interface::Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        *self.inbound.lock().unwrap() = Some(tx);
        Ok(rx)
    }

    async fn send(&self, data: &[u8], _path: &str) -> Result<(), jarust_interface::Error> {
        let request: Value = serde_json::from_slice(data).unwrap();
        let transaction = request["transaction"].as_str().unwrap().to_string();
        match request["janus"].as_str().unwrap() {
            "create" => self.reply(json!({
                "janus": "success",
                "transaction": transaction,
                "data": { "id": self.session_id }
            })),
            "attach" => self.reply(json!({
                "janus": "success",
                "transaction": transaction,
                "data": { "id": self.handle_id }
            })),
            "keepalive" => self.reply(json!({
                "janus": "ack",
                "transaction": transaction
            })),
            "destroy" => self.reply(json!({
                "janus": "success",
                "transaction": transaction,
                "data": { "id": self.session_id }
            })),
            other => panic!("unexpected janus action: {other}"),
        }
        Ok(())
    }
}

fn conn_params() -> ConnectionParams {
    ConnectionParams {
        url: "loopback://test".to_string(),
        capacity: 32,
        apisecret: None,
        server_root: "janus".to_string(),
    }
}

#[tokio::test]
async fn create_attach_keepalive_destroy_roundtrip() {
    let session_id = 111;
    let handle_id = 222;
    let transport = LoopbackTransport::new(session_id, handle_id);
    let interface = CustomInterface::new(transport, conn_params(), RandomTransactionGenerator)
        .await
        .expect("interface builds");

    let created = interface.create(TIMEOUT).await.expect("create");
    assert_eq!(created, session_id);

    let (attached, _events) = interface
        .attach(session_id, "janus.plugin.echotest".to_string(), TIMEOUT)
        .await
        .expect("attach");
    assert_eq!(attached, handle_id);

    assert!(interface.has_keep_alive());
    interface
        .keep_alive(session_id, TIMEOUT)
        .await
        .expect("keep_alive");

    interface.destroy(session_id, TIMEOUT).await.expect("destroy");
}

#[tokio::test]
async fn make_interface_is_unsupported() {
    let result =
        CustomInterface::make_interface(conn_params(), RandomTransactionGenerator).await;
    assert!(result.is_err(), "make_interface must not build a CustomInterface");
}
