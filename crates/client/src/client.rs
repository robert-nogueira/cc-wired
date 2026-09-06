use crate::ws::WsClient;

pub struct Client {
    pub ws: WsClient,
}

impl Client {
    pub async fn new(
        ws: WsClient,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { ws })
    }

    // that will be changed
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(message) = self.ws.recv().await {
            let message = message?;

            println!("Received: {message:?}");
        }

        Ok(())
    }
}
