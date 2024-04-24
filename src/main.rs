use clap::Parser;

use rcli::{process_csv, process_genpass, Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Commands::Csv(opts) => {
            let output = if let Some(output) = opts.output {
                output.clone()
            } else {
                format!("output.{}", opts.format)
            };
            let format = opts.format.to_string();
            process_csv(&opts.input, &output, &format)?;
        }
        Commands::GenPass(opts) => {
            process_genpass(
                opts.length,
                opts.lower,
                opts.upper,
                opts.digits,
                opts.symbol,
            )?;
        }
    }
    Ok(())
}
