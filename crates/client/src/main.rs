use cc_wired_client::{Error, Settings};

#[tokio::main]
async fn main() -> Result<(), Error> {
    cc_wired_client::run(Settings::load(None).expect("Invalid config")).await
}
