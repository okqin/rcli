use std::{net::IpAddr, path::PathBuf};

use clap::{Args, Subcommand};

use crate::{process_http_serve, CmdExecutor};

use super::{validate_addr, validate_path, validate_port};

#[derive(Debug, Subcommand)]
pub enum HttpCommand {
    /// Start a http server
    #[command(name = "serve")]
    Serve(HttpServerOpts),
    // /// Stop a http server
    // #[command(name = "stop")]
    // Stop(HttpServerOpts),
}

#[derive(Debug, Args)]
pub struct HttpServerOpts {
    /// the server address, default: 127.0.0.1
    #[arg(short, long, value_parser = validate_addr, default_value = "127.0.0.1")]
    pub addr: IpAddr,

    /// the server port, default: 8080
    #[arg(short, long, value_parser = validate_port, default_value = "8080")]
    pub port: u16,

    /// file service root path
    #[arg(long, value_parser = validate_path)]
    pub path: PathBuf,

    /// whether to start as a daemon
    #[arg(short, long)]
    pub daemon: bool,
}

impl CmdExecutor for HttpServerOpts {
    async fn execute(self) -> anyhow::Result<()> {
        process_http_serve(self.path, &self.addr, self.port, self.daemon).await?;
        Ok(())
    }
}

impl CmdExecutor for HttpCommand {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            HttpCommand::Serve(opts) => opts.execute().await?,
            // HttpCommand::Stop(opts) => opts.execute().await,
        }
        Ok(())
    }
}
