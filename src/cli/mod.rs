mod base64;
mod csv;
mod genpass;
mod http;
mod jwt;
mod text;

pub use self::{base64::*, csv::*, genpass::*, http::*, jwt::*, text::*};
use chrono::Utc;
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

    /// jwt sign or verify
    #[command(subcommand, name = "jwt")]
    Jwt(JwtCommand),
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

fn validate_exp_time(s: &str) -> Result<u64, String> {
    let mut parts = s.chars().peekable();
    let mut duration_str = String::new();
    while let Some(&ch) = parts.peek() {
        if ch.is_ascii_digit() {
            duration_str.push(ch);
            parts.next();
        } else {
            break;
        }
    }
    let duration = duration_str
        .parse::<i64>()
        .map_err(|_| format!("`{}` isn't a valid numbers", s))?;
    let unit = parts
        .take_while(|c| c.is_ascii_alphabetic())
        .collect::<String>();
    let seconds = match unit.as_str() {
        "s" => duration,
        "m" => duration * 60,
        "h" => duration * 3600,
        "d" => duration * 86400,
        "w" => duration * 604800,
        "M" => duration * 2592000,
        "y" => duration * 31536000,
        _ => {
            return Err(format!(
                "`{}` isn't a valid time unit,[s, m, h, d, w, M, y]",
                s
            ))
        }
    };
    let timestamp = Utc::now().timestamp() + seconds;
    Ok(timestamp as u64)
}

#[test]
fn test_validate_exp_time() {
    assert_eq!(
        validate_exp_time("12s").unwrap(),
        (Utc::now().timestamp() + 12) as u64
    );
}
