use std::fmt;

use clap::{Args, ValueEnum};

use super::validate_input_file;

#[derive(Debug, Args)]
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

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Yaml => write!(f, "yaml"),
        }
    }
}
