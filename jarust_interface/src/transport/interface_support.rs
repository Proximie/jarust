//! Transport-agnostic helpers shared by the [`JanusInterface`] implementations.
//!
//! [`JanusInterface`]: crate::janus_interface::JanusInterface

use crate::japrotocol::JaResponse;
use crate::japrotocol::JaSuccessProtocol;
use crate::japrotocol::ResponseType;
use crate::tgenerator::TransactionGenerator;
use crate::transport::napmap::NapMap;
use crate::Error;
use serde_json::Value;
use std::time::Duration;

/// Injects the optional `apisecret` and a freshly generated `transaction` into a
/// request, returning the decorated request and the transaction id.
pub(crate) fn decorate_request(
    generator: &TransactionGenerator,
    apisecret: Option<&str>,
    mut request: Value,
) -> (Value, String) {
    let transaction = generator.generate_transaction();
    if let Some(apisecret) = apisecret {
        request["apisecret"] = apisecret.into();
    }
    request["transaction"] = transaction.clone().into();
    (request, transaction)
}

/// Waits for the response/ack carrying `transaction` to land in `map`, bounded by
/// `timeout`. Used by streaming transports (WebSocket, Socket.IO) that demultiplex
/// inbound frames into response and ack maps keyed by transaction.
pub(crate) async fn poll_transaction(
    map: &NapMap<String, JaResponse>,
    transaction: &str,
    timeout: Duration,
) -> Result<JaResponse, Error> {
    match tokio::time::timeout(timeout, map.get(transaction.to_string())).await {
        Ok(Some(response)) => match response.janus {
            ResponseType::Error { error } => Err(Error::JanusError {
                code: error.code,
                reason: error.reason,
            }),
            _ => Ok(response),
        },
        Ok(None) => {
            tracing::error!("Incomplete packet");
            Err(Error::IncompletePacket)
        }
        Err(_) => {
            tracing::error!("Request timeout");
            Err(Error::RequestTimeout)
        }
    }
}

/// Extracts the `id` from a `create`/`attach` success response, mapping Janus errors
/// and unexpected shapes to the appropriate [`Error`].
// The small `Ok(u64)` next to the (crate-wide) large `Error` enum trips
// `result_large_err`; boxing the error is a broader API change tracked separately, and
// every `JanusInterface` method already returns this same `Error` by value.
#[allow(clippy::result_large_err)]
pub(crate) fn extract_id(response: JaResponse) -> Result<u64, Error> {
    match response.janus {
        ResponseType::Success(JaSuccessProtocol::Data { data }) => Ok(data.id),
        ResponseType::Error { error } => {
            let what = Error::JanusError {
                code: error.code,
                reason: error.reason,
            };
            tracing::error!("{what}");
            Err(what)
        }
        _ => {
            tracing::error!("Unexpected response");
            Err(Error::UnexpectedResponse)
        }
    }
}

/// Recursively merges JSON object `b` into `a`. Non-object values in `b` overwrite the
/// corresponding value in `a`.
pub(crate) fn merge_json(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), Value::Object(b)) => {
            for (k, v) in b {
                merge_json(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}
