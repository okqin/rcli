use std::{
    fs,
    io::{self, Read},
    path::Path,
};

use anyhow::Result;
use base64::{
    alphabet::{STANDARD, URL_SAFE},
    engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig},
    Engine,
};

pub const CUSTOM_PAD: GeneralPurposeConfig =
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent);

pub const CUSTOM_NO_PAD: GeneralPurposeConfig = GeneralPurposeConfig::new()
    .with_encode_padding(false)
    .with_decode_padding_mode(DecodePaddingMode::Indifferent);

pub const STANDARD_ENGINE: GeneralPurpose = GeneralPurpose::new(&STANDARD, CUSTOM_PAD);

pub const URL_SAFE_ENGINE: GeneralPurpose = GeneralPurpose::new(&URL_SAFE, CUSTOM_NO_PAD);

pub fn process_encode(input: &str, format: &str) -> Result<String> {
    let mut reader = get_reader(input)?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    let result = match format {
        "url" => URL_SAFE_ENGINE.encode(&buf),
        _ => STANDARD_ENGINE.encode(&buf),
    };
    Ok(result)
}

pub fn process_decode(input: &str, format: &str) -> Result<String> {
    let mut reader = get_reader(input)?;
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    let buf = buf.trim();
    let decoded = match format {
        "url" => URL_SAFE_ENGINE.decode(buf)?,
        _ => STANDARD_ENGINE.decode(buf)?,
    };

    match String::from_utf8(decoded.clone()) {
        Ok(result) => Ok(result),
        Err(_) => {
            let file = Path::new("base64_decode.output");
            fs::write(file, decoded)?;
            Ok(format!(
                "The decode data is not a string, please check the file {}",
                file.display()
            ))
        }
    }
}

fn get_reader(input: &str) -> Result<Box<dyn Read>> {
    let reader: Box<dyn Read> = if input == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(fs::File::open(input)?)
    };
    Ok(reader)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_encode_standard() {
        let input = "assets/decode.b64";
        let format = "standard";
        let encoded = process_encode(input, format).unwrap();
        assert_eq!(encoded, "VGhpcyBpcyBhIGJhc2U2NCBlbmNvZGluZyB0ZXh0Lgo=");
    }

    #[test]
    fn test_process_encode_url() {
        let input = "assets/decode.b64";
        let format = "url";
        let encoded = process_encode(input, format).unwrap();
        assert_eq!(encoded, "VGhpcyBpcyBhIGJhc2U2NCBlbmNvZGluZyB0ZXh0Lgo");
    }

    #[test]
    fn test_process_decode_standard() {
        let input = "assets/encode.b64";
        let format = "standard";
        let decoded = process_decode(input, format).unwrap();
        assert_eq!(decoded, "This is a base64 encoding text.");
    }

    #[test]
    fn test_process_decode_url() {
        let input = "assets/encode.b64";
        let format = "url";
        let decoded = process_decode(input, format).unwrap();
        assert_eq!(decoded, "This is a base64 encoding text.");
    }

    #[test]
    fn test_process_decode_not_string() {
        let input = "assets/encode-png.b64";
        let format = "standard";
        let decoded = process_decode(input, format).unwrap();
        assert_eq!(
            decoded,
            "The decode data is not a string, please check the file base64_decode.output"
        );
    }
}
