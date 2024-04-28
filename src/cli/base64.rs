use std::fmt;

use clap::{Args, Subcommand, ValueEnum};

use super::validate_input_file;

#[derive(Debug, Subcommand)]
pub enum Base64Command {
    /// Base64 encode
    #[command(name = "encode")]
    Encode(Base64EncodeOpts),

    /// Base64 decode
    #[command(name = "decode")]
    Decode(Base64DecodeOpts),
}

#[derive(Debug, Args)]
pub struct Base64EncodeOpts {
    /// input a string or file to encode
    #[arg(short, long, value_parser = validate_input_file, default_value = "-")]
    pub input: String,

    /// encode format, like: standard or url (default: standard)
    #[arg(long, value_enum, default_value = "standard")]
    pub format: AlphabetRange,
}

#[derive(Debug, Args)]
pub struct Base64DecodeOpts {
    /// input from stdin or file to decode/encode
    #[arg(short, long, value_parser = validate_input_file, default_value = "-")]
    pub input: String,

    /// base64 format, like: standard or url (default: standard)
    #[arg(long, value_enum, default_value = "standard")]
    pub format: AlphabetRange,
}

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum AlphabetRange {
    /// standard base64 alphabet
    Standard,

    /// url safe base64 alphabet
    Url,
}

impl fmt::Display for AlphabetRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlphabetRange::Standard => write!(f, "standard"),
            AlphabetRange::Url => write!(f, "url"),
        }
    }
}
