use anyhow::Result;
use std::{
    fs,
    io::{self, Read},
};

pub fn get_reader(input: &str) -> Result<Box<dyn Read>> {
    let reader: Box<dyn Read> = if input == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(fs::File::open(input)?)
    };
    Ok(reader)
}
