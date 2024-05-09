use crate::{process_genpass, CmdExecutor};
use clap::Args;
use zxcvbn::zxcvbn;

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

impl CmdExecutor for GenPassOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let pass = process_genpass(
            self.length,
            self.lower,
            self.upper,
            self.digits,
            self.symbol,
        )?;
        unsafe {
            let password = String::from_utf8_unchecked(pass);
            println!("{}", password);
            let estimate = zxcvbn(&password, &[])?;
            eprintln!("Estimated strength: {}\n", estimate.score());
        }
        Ok(())
    }
}
