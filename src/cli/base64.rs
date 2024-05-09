use super::validate_file;
use crate::{process_decode, process_encode, CmdExecutor};
use clap::{Args, Subcommand, ValueEnum};
use std::{fmt, fs, path::Path};

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
    /// input from stdin or file to encode
    #[arg(short, long, value_parser = validate_file, default_value = "-")]
    pub input: String,

    /// base64 format, like: standard or url (default: standard)
    #[arg(long, value_enum, default_value = "standard")]
    pub format: AlphabetRange,
}

#[derive(Debug, Args)]
pub struct Base64DecodeOpts {
    /// input from stdin or file to decode
    #[arg(short, long, value_parser = validate_file, default_value = "-")]
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

impl CmdExecutor for Base64EncodeOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let encoded = process_encode(&self.input, &self.format.to_string())?;
        println!("{}", encoded);
        Ok(())
    }
}

impl CmdExecutor for Base64DecodeOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let decoded = process_decode(&self.input, &self.format.to_string())?;
        match String::from_utf8(decoded.clone()) {
            Ok(result) => println!("{}", result),
            Err(_) => {
                let file = Path::new("base64_decode.output");
                fs::write(file, decoded)?;
                println!(
                    "The decode data is not a string, please check the file {}",
                    file.display()
                )
            }
        }
        Ok(())
    }
}

impl CmdExecutor for Base64Command {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            Base64Command::Encode(opts) => opts.execute().await,
            Base64Command::Decode(opts) => opts.execute().await,
        }
    }
}

impl fmt::Display for AlphabetRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlphabetRange::Standard => write!(f, "standard"),
            AlphabetRange::Url => write!(f, "url"),
        }
    }
}
