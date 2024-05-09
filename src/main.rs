use clap::Parser;
use rcli::{Cli, CmdExecutor};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.cmd.execute().await?;
    Ok(())
}
