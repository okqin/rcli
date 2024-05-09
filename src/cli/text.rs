use super::{validate_file, validate_path};
use crate::{
    process_text_generate_key, process_text_sign, process_text_verify, CmdExecutor, URL_SAFE_ENGINE,
};
use base64::Engine;
use clap::{Args, Subcommand, ValueEnum};
use std::{fmt, fs, path::PathBuf};

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

impl CmdExecutor for TextSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let signature = process_text_sign(&self.message, &self.key, &self.format.to_string())?;
        let signature = URL_SAFE_ENGINE.encode(signature);
        println!("{}", signature);
        Ok(())
    }
}

impl CmdExecutor for TextVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let result = process_text_verify(
            &self.message,
            &self.key,
            &self.format.to_string(),
            self.signature.as_bytes(),
        )?;
        println!("{}", result);
        Ok(())
    }
}

impl CmdExecutor for TextGenerateKeyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let key = process_text_generate_key(&self.format.to_string())?;
        let path = self.output;
        match self.format {
            SignFormat::Blake3 => {
                fs::write(path.join("blake3.txt"), key[0])?;
            }
            SignFormat::Ed25519 => {
                fs::write(path.join("ed25519.sk"), key[0])?;
                fs::write(path.join("ed25519.pk"), key[1])?;
            }
        }
        Ok(())
    }
}

impl CmdExecutor for TextCommand {
    async fn execute(self) -> anyhow::Result<()> {
        match self {
            TextCommand::Sign(opts) => opts.execute().await?,
            TextCommand::Verify(opts) => opts.execute().await?,
            TextCommand::GenerateKey(opts) => opts.execute().await?,
        }
        Ok(())
    }
}

impl fmt::Display for SignFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignFormat::Blake3 => write!(f, "blake3"),
            SignFormat::Ed25519 => write!(f, "ed25519"),
        }
    }
}
