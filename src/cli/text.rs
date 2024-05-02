use std::{fmt, path::PathBuf};

use clap::{Args, Subcommand, ValueEnum};

use super::{validate_file, validate_path};

#[derive(Debug, Subcommand)]
pub enum TextCommand {
    /// Sign a message with a key file
    #[command(name = "sign")]
    Sign(TextSignOpts),

    /// Verify a signed message with a key file
    #[command(name = "verify")]
    Verify(TextVerifyOpts),

    /// Generate a hash key or an asymmetric key pair.
    #[command(name = "gen")]
    GenerateKey(TextGenerateKeyOpts),
}

#[derive(Debug, Args)]
pub struct TextSignOpts {
    /// a message to signing, from file or stdin
    #[arg(short, long, value_parser = validate_file, default_value = "-")]
    pub message: String,

    /// the sign key file, like: secret key
    #[arg(short, long, value_parser = validate_file)]
    pub key: String,

    /// the signature format
    #[arg(long, value_enum, default_value = "blake3")]
    pub format: SignFormat,
}

#[derive(Debug, Args)]
pub struct TextVerifyOpts {
    /// a message to be verified, from file or stdin
    #[arg(short, long, value_parser = validate_file, default_value = "-")]
    pub message: String,

    /// the verify key file, like: public key
    #[arg(short, long, value_parser = validate_file)]
    pub key: String,

    /// the signature format
    #[arg(long, value_enum, default_value = "blake3")]
    pub format: SignFormat,

    /// the signature
    #[arg(short, long)]
    pub signature: String,
}

#[derive(Debug, Args)]
pub struct TextGenerateKeyOpts {
    /// the key type
    #[arg(long, value_enum, default_value = "blake3")]
    pub format: SignFormat,

    /// save the key to a dir
    #[arg(short, long, value_parser = validate_path)]
    pub output: PathBuf,
}

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum SignFormat {
    /// blake3 signature
    Blake3,

    /// ed25519 signature
    Ed25519,
}

impl fmt::Display for SignFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignFormat::Blake3 => write!(f, "blake3"),
            SignFormat::Ed25519 => write!(f, "ed25519"),
        }
    }
}
