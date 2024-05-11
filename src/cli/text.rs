use super::{validate_file, validate_path, CmdExecutor};
use crate::{
    get_reader, process_text_decrypt, process_text_encrypt, process_text_generate_key,
    process_text_sign, process_text_verify, read_contents, URL_SAFE_ENGINE,
};
use anyhow::{anyhow, Result};
use base64::Engine;
use clap::{Args, Subcommand, ValueEnum};
use enum_dispatch::enum_dispatch;
use std::{fmt, fs, path::PathBuf};

#[enum_dispatch(CmdExecutor)]
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

    /// Encrypt a message with a key file
    #[command(name = "encrypt")]
    Encrypt(TextEncryptOpts),

    /// Decrypt a message with a key file
    #[command(name = "decrypt")]
    Decrypt(TextDecryptOpts),
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

#[derive(Debug, Args)]
pub struct TextEncryptOpts {
    /// a message to encrypt, from file or stdin
    #[arg(short, long, value_parser = validate_file, default_value = "-")]
    pub message: String,

    /// the encrypt key file
    #[arg(short, long, value_parser = validate_file)]
    pub key: String,

    /// the cipher kind
    #[arg(long, value_enum, default_value = "chacha20-poly1305")]
    pub cipher: CipherKind,
}

#[derive(Debug, Args)]
pub struct TextDecryptOpts {
    /// a message to decrypt, from file or stdin
    #[arg(short, long, value_parser = validate_file, default_value = "-")]
    pub message: String,

    /// the decrypt key file
    #[arg(short, long, value_parser = validate_file)]
    pub key: String,

    /// the cipher kind
    #[arg(long, value_enum, default_value = "chacha20-poly1305")]
    pub cipher: CipherKind,
}

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum CipherKind {
    /// chacha20poly1305 algorithm
    Chacha20Poly1305,
}

impl CmdExecutor for TextSignOpts {
    async fn execute(self) -> Result<()> {
        let mut message = get_reader(&self.message)?;
        let key = read_contents(&self.key)?;
        let signature = process_text_sign(&mut message, &key, &self.format.to_string())?;
        let encoded = URL_SAFE_ENGINE.encode(signature);
        println!("{}", encoded);
        Ok(())
    }
}

impl CmdExecutor for TextVerifyOpts {
    async fn execute(self) -> Result<()> {
        let mut message = get_reader(&self.message)?;
        let key = read_contents(&self.key)?;
        let result = process_text_verify(
            &mut message,
            &key,
            &self.format.to_string(),
            self.signature.as_bytes(),
        )?;
        println!("{}", result);
        Ok(())
    }
}

impl CmdExecutor for TextGenerateKeyOpts {
    async fn execute(self) -> Result<()> {
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

impl CmdExecutor for TextEncryptOpts {
    async fn execute(self) -> Result<()> {
        let message = read_contents(&self.message)?;
        let key = read_contents(&self.key)?;
        let encrypted = process_text_encrypt(&message, &key, &self.cipher.to_string())?;
        let encoded = URL_SAFE_ENGINE.encode(encrypted);
        println!("{}", encoded);
        Ok(())
    }
}

impl CmdExecutor for TextDecryptOpts {
    async fn execute(self) -> Result<()> {
        let message = read_contents(&self.message)?;
        let decode = URL_SAFE_ENGINE.decode(message).map_err(|e| {
            anyhow!("base64 decode error: {e} perhaps you could check the file for line breaks.")
        })?;
        let key = read_contents(&self.key)?;
        let decrypted = process_text_decrypt(&decode, &key, &self.cipher.to_string())?;
        let plaintext = String::from_utf8(decrypted)?;
        println!("{}", plaintext);
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

impl fmt::Display for CipherKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CipherKind::Chacha20Poly1305 => write!(f, "chacha20poly1305"),
        }
    }
}
