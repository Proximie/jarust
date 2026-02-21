use crate::Error;
use rustls::RootCertStore;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::connect_async_tls_with_config;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::Connector;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

fn make_tls_client_config() -> Result<Arc<rustls::ClientConfig>, Error> {
    let mut root_store = RootCertStore::empty();
    let platform_certs = rustls_native_certs::load_native_certs().certs;
    root_store.add_parsable_certificates(platform_certs);
    let client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    Ok(Arc::new(client_config))
}

pub async fn connect_async(
    request: Request,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Error> {
    tracing::trace!("Using rustls");
    let connector = Connector::Rustls(make_tls_client_config()?);
    let (stream, ..) = connect_async_tls_with_config(request, None, true, Some(connector)).await?;
    Ok(stream)
}
