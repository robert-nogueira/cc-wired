use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};

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
    pub async fn connect(addr: String) -> Result<Self, WsError> {
        let (stream, _response) =
            tokio_tungstenite::connect_async(addr).await?;
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
