mod watch;

use anyhow::Result;
use async_trait::async_trait;

use clap::Subcommand;
use watch::Watch;

#[derive(Subcommand)]
pub enum Commands {
    Watch(Watch),
}

impl Commands {
    pub async fn run(&self) -> Result<()> {
        match self {
            Self::Watch(cmd) => cmd.run().await,
        }
    }
}

#[async_trait]
pub trait Command {
    async fn run(&self) -> Result<()>;
}
