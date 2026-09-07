use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};

use tokio_tungstenite::tungstenite::http::Uri;

use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, tungstenite::Message,
};

type WsSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WsSource = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub type WsError = tokio_tungstenite::tungstenite::Error;

pub struct WsClient {
    sink: WsSink,
    source: WsSource,
}

impl WsClient {
    pub async fn connect(
        addr: String,
        timeout: std::time::Duration,
    ) -> Result<Self, WsError> {
        let uri: Uri = addr.parse()?;
        let client_id = uuid::Uuid::new_v4();
        let builder =
            tokio_tungstenite::tungstenite::ClientRequestBuilder::new(uri)
                .with_header("x-client-id", client_id);

        let (stream, _response) = tokio::time::timeout(
            timeout,
            tokio_tungstenite::connect_async(builder),
        )
        .await
        .map_err(|_| {
            WsError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "web socket connection timed out",
            ))
        })??;

        let (sink, source) = stream.split();

        Ok(Self { sink, source })
    }

    pub async fn send(&mut self, message: Message) -> Result<(), WsError> {
        self.sink.send(message).await
    }

    pub async fn recv(&mut self) -> Option<Result<Message, WsError>> {
        self.source.next().await
    }

    pub async fn close(mut self) -> Result<(), WsError> {
        self.sink.close().await?;

        while let Some(message) = self.source.next().await {
            if let Message::Close(_) = message? {
                break;
            }
        }

        Ok(())
    }
}
