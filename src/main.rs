use clap::Parser;

use rcli::{
    process_csv, process_decode, process_encode, process_genpass, Base64Command, Cli, Commands,
};

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
        Commands::Base64(subcommand) => match subcommand {
            Base64Command::Encode(opts) => {
                let encoded = process_encode(&opts.input, &opts.format.to_string())?;
                println!("{}", encoded);
            }
            Base64Command::Decode(opts) => {
                let decoded = process_decode(&opts.input, &opts.format.to_string())?;
                println!("{}", decoded);
            }
        },
    }
    Ok(())
}
