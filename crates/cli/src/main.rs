use anyhow::Result;
use cc_wired::commands::Commands;
use clap::Parser;

#[derive(Parser)]
#[command(name = "cc-wired")]
#[command(next_line_help = true)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    args.command.run().await
}
