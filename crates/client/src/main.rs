use cc_wired_client::Settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cc_wired_client::run(Settings::load().expect("Invalid config")).await
}
