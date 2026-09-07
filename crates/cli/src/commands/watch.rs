use super::Command;

use anyhow::Result;
use async_trait::async_trait;
use cc_wired_client::Settings;
use clap::Parser;

#[derive(Parser)]
pub struct Watch {
    #[arg(short, long, default_value = "config")]
    config: std::path::PathBuf,
}

#[async_trait]
impl Command for Watch {
    async fn run(&self) -> Result<()> {
        let settings =
            Settings::load(self.config.with_extension("").to_str())?;

        cc_wired_client::run(settings).await?;

        Ok(())
    }
}
