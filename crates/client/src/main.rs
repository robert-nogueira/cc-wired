#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cc_wired_client::run().await
}
