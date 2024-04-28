mod base64;
mod csv;
mod genpass;

use std::path::Path;

pub use self::{base64::Base64Command, csv::CsvOpts, genpass::GenPassOpts};

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "rcli", version, author, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Show Csv or Convert CSV to other formats
    #[command(name = "csv")]
    Csv(CsvOpts),

    /// Generate a random password
    #[command(name = "genpass")]
    GenPass(GenPassOpts),

    /// Encode or Decode a base64 string
    #[command(subcommand, name = "base64")]
    Base64(Base64Command),
}

fn validate_input_file(filename: &str) -> Result<String, String> {
    if filename == "-" || Path::new(filename).exists() {
        Ok(filename.to_string())
    } else {
        Err(format!("File not found: {}", filename))
    }
}
