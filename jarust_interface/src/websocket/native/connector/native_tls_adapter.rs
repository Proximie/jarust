use crate::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::connect_async_with_config;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

pub async fn connect_async(
    request: Request,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Error> {
    tracing::trace!("Using native-tls");
    let (stream, ..) = connect_async_with_config(request, None, false).await?;
    Ok(stream)
}
