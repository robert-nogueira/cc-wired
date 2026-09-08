use super::Command;

use anyhow::Result;
use async_trait::async_trait;
use cc_wired_server::settings::Settings;
use clap::Parser;

#[derive(Parser)]
pub struct Server {
    #[arg(long)]
    host: Option<String>,

    #[arg(short, long)]
    port: Option<u16>,
}

#[async_trait]
impl Command for Server {
    async fn run(&self) -> Result<()> {
        let mut settings = Settings::load(None)?;

        if let Some(host) = &self.host {
            settings.host = host.clone();
        }
        if let Some(port) = self.port {
            settings.port = port;
        }
        cc_wired_server::run(settings).await;

        Ok(())
    }
}
