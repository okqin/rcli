use super::{validate_exp_time, CmdExecutor};
use crate::{process_jwt_sign_with_secret, process_jwt_verify_with_secret};
use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};
use core::fmt;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[enum_dispatch(CmdExecutor)]
#[derive(Debug, Subcommand)]
pub enum JwtCommand {
    /// Generate a jwt for the given payload.
    #[command(name = "sign")]
    Sign(JwtSignOpts),

    /// Verify a jwt token with a shared secret.
    #[command(name = "verify")]
    Verify(JwtVerifyOpts),
}

#[derive(Debug, Args)]
pub struct JwtSignOpts {
    /// the payload to be signed
    #[command(flatten)]
    pub payload: Payload,

    /// the sign secret
    #[arg(short, long)]
    pub key: String,

    /// the signature algorithm
    #[arg(long, value_enum, default_value = "hs256")]
    pub alg: JwtAlgorithm,
}

#[derive(Debug, Args)]
pub struct JwtVerifyOpts {
    /// the jwt token to be verified
    #[arg(short, long)]
    pub token: String,

    /// the verify secret
    #[arg(short, long)]
    pub key: String,

    /// the signature algorithm
    #[arg(long, value_enum)]
    pub alg: Option<JwtAlgorithm>,
}

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum JwtAlgorithm {
    /// HMAC SHA256 algorithm
    HS256,
}

#[derive(Debug, Serialize, Deserialize, Args)]
pub struct Payload {
    /// the subject field
    #[arg(long)]
    pub sub: String,

    /// the audience field
    #[arg(long)]
    pub aud: String,

    /// the expiration time field, like, 1m, 1h, 1d, 1w, 1M
    #[arg(long, value_parser = validate_exp_time)]
    pub exp: u64,
}

impl CmdExecutor for JwtSignOpts {
    async fn execute(self) -> Result<()> {
        let token = process_jwt_sign_with_secret(
            &self.payload,
            self.key.as_bytes(),
            &self.alg.to_string(),
        )?;
        println!("{}", token);
        Ok(())
    }
}

impl CmdExecutor for JwtVerifyOpts {
    async fn execute(self) -> Result<()> {
        let data = process_jwt_verify_with_secret::<Payload>(
            &self.token,
            self.key.as_bytes(),
            self.alg.as_deref(),
        )?;
        println!("{:?}", data);
        Ok(())
    }
}

impl fmt::Display for JwtAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JwtAlgorithm::HS256 => write!(f, "HS256"),
        }
    }
}

impl Deref for JwtAlgorithm {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            JwtAlgorithm::HS256 => "HS256",
        }
    }
}
