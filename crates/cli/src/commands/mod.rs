mod server;
mod watch;

use anyhow::Result;
use async_trait::async_trait;

use clap::Subcommand;
use server::Server;
use watch::Watch;

#[derive(Subcommand)]
pub enum Commands {
    Watch(Watch),
    Server(Server),
}

impl Commands {
    pub async fn run(&self) -> Result<()> {
        match self {
            Self::Watch(cmd) => cmd.run().await,
            Self::Server(cmd) => cmd.run().await,
        }
    }
}

#[async_trait]
pub trait Command {
    async fn run(&self) -> Result<()>;
}
