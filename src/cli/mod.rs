mod base64;
mod csv;
mod genpass;
mod http;
mod text;

pub use self::{base64::*, csv::*, genpass::*, http::*, text::*};
use clap::{Parser, Subcommand};
use enum_dispatch::enum_dispatch;
use std::{
    net::IpAddr,
    ops::RangeInclusive,
    path::{Path, PathBuf},
};

#[derive(Debug, Parser)]
#[command(name = "rcli", version, author, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Commands,
}

#[enum_dispatch(CmdExecutor)]
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

#[allow(async_fn_in_trait)]
#[enum_dispatch]
pub trait CmdExecutor {
    async fn execute(self) -> anyhow::Result<()>;
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
