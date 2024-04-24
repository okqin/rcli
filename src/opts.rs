use core::str;
use std::{fmt, path::Path};

use clap::{Parser, Subcommand, ValueEnum};

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
}

#[derive(Debug, Parser)]
pub struct CsvOpts {
    /// Input CSV file path
    #[arg(short, long, value_parser = validate_input_file)]
    pub input: String,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,

    /// Output file format
    #[arg(long, value_enum, default_value = "json")]
    pub format: OutputFormat,

    /// Delimiter used in CSV file
    #[arg(short, long, default_value_t = ',')]
    pub delimiter: char,

    /// Whether to include header in output
    #[arg(long, default_value_t = true)]
    pub header: bool,
}

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum OutputFormat {
    /// output json format
    Json,

    /// output yaml format
    Yaml,
}

#[derive(Debug, Parser)]
pub struct GenPassOpts {
    /// Length of the password
    #[arg(short, long, default_value_t = 16)]
    pub length: u8,

    /// Whether to include lowercase characters
    #[arg(long, default_value_t = true)]
    pub lower: bool,

    /// Whether to include uppercase characters
    #[arg(long, default_value_t = true)]
    pub upper: bool,

    /// Whether to include digits
    #[arg(long, default_value_t = true)]
    pub digits: bool,

    /// Whether to include symbols
    #[arg(long, default_value_t = true)]
    pub symbol: bool,
}

fn validate_input_file(filename: &str) -> Result<String, String> {
    if Path::new(filename).exists() {
        Ok(filename.to_string())
    } else {
        Err(format!("File not found: {}", filename))
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Yaml => write!(f, "yaml"),
        }
    }
}
