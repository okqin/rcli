mod base64;
mod csv;
mod genpass;
mod text;

use std::path::{Path, PathBuf};

pub use self::{
    base64::Base64Command, csv::CsvOpts, genpass::GenPassOpts, text::SignFormat, text::TextCommand,
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
