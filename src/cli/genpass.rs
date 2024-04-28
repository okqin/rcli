use clap::Args;

#[derive(Debug, Args)]
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
