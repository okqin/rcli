use clap::Parser;

use rcli::{process_csv, Cli, SubCommand};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        SubCommand::Csv(opts) => {
            let output = if let Some(output) = opts.output {
                output.clone()
            } else {
                format!("output.{}", opts.format)
            };
            process_csv(&opts.input, &output, opts.format)?;
        }
    }
    Ok(())
}
