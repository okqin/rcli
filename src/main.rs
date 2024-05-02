use std::{fs, path::Path};

use base64::Engine;
use clap::Parser;
use zxcvbn::zxcvbn;

use rcli::{
    process_csv, process_decode, process_encode, process_genpass, process_text_generate_key,
    process_text_sign, process_text_verify, Base64Command, Cli, Commands, SignFormat, TextCommand,
    URL_SAFE_ENGINE,
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
            let password = process_genpass(
                opts.length,
                opts.lower,
                opts.upper,
                opts.digits,
                opts.symbol,
            )?;
            unsafe {
                let password = String::from_utf8_unchecked(password);
                println!("{}", password);
                let estimate = zxcvbn(&password, &[])?;
                eprintln!("Estimated strength: {}\n", estimate.score());
            }
        }
        Commands::Base64(subcommand) => match subcommand {
            Base64Command::Encode(opts) => {
                let encoded = process_encode(&opts.input, &opts.format.to_string())?;
                println!("{}", encoded);
            }
            Base64Command::Decode(opts) => {
                let decoded = process_decode(&opts.input, &opts.format.to_string())?;
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
            }
        },
        Commands::Text(subcommand) => match subcommand {
            TextCommand::Sign(opts) => {
                let signature =
                    process_text_sign(&opts.message, &opts.key, &opts.format.to_string())?;
                let signature = URL_SAFE_ENGINE.encode(signature);
                println!("{}", signature);
            }
            TextCommand::Verify(opts) => {
                let result = process_text_verify(
                    &opts.message,
                    &opts.key,
                    &opts.format.to_string(),
                    opts.signature.as_bytes(),
                )?;
                println!("{}", result);
            }
            TextCommand::GenerateKey(opts) => {
                let key = process_text_generate_key(&opts.format.to_string())?;
                let path = opts.output;
                match opts.format {
                    SignFormat::Blake3 => {
                        fs::write(path.join("blake3.txt"), key[0])?;
                    }
                    SignFormat::Ed25519 => {
                        fs::write(path.join("ed25519.sk"), key[0])?;
                        fs::write(path.join("ed25519.pk"), key[1])?;
                    }
                }
            }
        },
    }
    Ok(())
}
