use cc_wired_server::settings;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let settings = settings::Settings::load(None).expect("Invalid config");
    cc_wired_server::run(settings).await
}
