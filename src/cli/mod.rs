mod base64;
mod csv;
mod genpass;
mod http;
mod text;

use std::{
    net::IpAddr,
    ops::RangeInclusive,
    path::{Path, PathBuf},
};

use crate::CmdExecutor;

pub use self::{
    base64::Base64Command, csv::CsvOpts, genpass::GenPassOpts, http::HttpCommand, text::SignFormat,
    text::TextCommand,
};

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "rcli", version, author, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Convert csv to other formats
    #[command(name = "csv")]
    Csv(CsvOpts),

    /// Generate a random password
    #[command(name = "genpass")]
    GenPass(GenPassOpts),

    /// Use base64 for encoding or decoding
    #[command(subcommand, name = "base64")]
    Base64(Base64Command),

    /// Text signing or signature verification.
    #[command(subcommand, name = "text")]
    Text(TextCommand),

    /// Start a simple file http server
    #[command(subcommand, name = "http")]
    Http(HttpCommand),
}

impl CmdExecutor for Commands {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            Commands::Csv(opts) => opts.execute().await?,
            Commands::GenPass(opts) => opts.execute().await?,
            Commands::Base64(subcommand) => subcommand.execute().await?,
            Commands::Text(subcommand) => subcommand.execute().await?,
            Commands::Http(subcommand) => subcommand.execute().await?,
        }
        Ok(())
    }
}

fn validate_file(filename: &str) -> Result<String, String> {
    if filename == "-" || Path::new(filename).exists() {
        Ok(filename.to_string())
    } else {
        Err(format!("File not found: {}", filename))
    }
}

fn validate_path(path: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(path);
    if p.exists() && p.is_dir() {
        Ok(p)
    } else {
        Err(format!("Path not found: {}", path))
    }
}

const PORT_RANGE: RangeInclusive<usize> = 1..=65535;

fn validate_port(s: &str) -> Result<u16, String> {
    let port: usize = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a port number"))?;
    if PORT_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(format!(
            "port not in range {}-{}",
            PORT_RANGE.start(),
            PORT_RANGE.end()
        ))
    }
}

fn validate_addr(s: &str) -> Result<IpAddr, String> {
    s.parse()
        .map_err(|_| format!("`{}` isn't a valid IP address", s))
}
