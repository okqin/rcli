use rand::{seq::SliceRandom, Rng};

use zxcvbn::zxcvbn;

const LOWER: &[u8] = b"abcdefghjklmnpqrstuvwxyz";
const UPPER: &[u8] = b"ABCDEFGHJKLMNOPQRSTUVWXYZ";
const DIGITS: &[u8] = b"123456789";
const SYMBOL: &[u8] = b"!@#$%^&*";

pub fn process_genpass(
    length: u8,
    lower: bool,
    upper: bool,
    digits: bool,
    symbol: bool,
) -> anyhow::Result<()> {
    let mut charset = Vec::new();
    let mut password = Vec::new();
    let mut rng = rand::thread_rng();
    if lower {
        charset.extend_from_slice(LOWER);
        password.push(*LOWER.choose(&mut rng).expect("LOWER won't be empty"));
    }
    if upper {
        charset.extend_from_slice(UPPER);
        password.push(*UPPER.choose(&mut rng).expect("UPPER won't be empty"));
    }
    if digits {
        charset.extend_from_slice(DIGITS);
        password.push(*DIGITS.choose(&mut rng).expect("DIGITS won't be empty"));
    }
    if symbol {
        charset.extend_from_slice(SYMBOL);
        password.push(*SYMBOL.choose(&mut rng).expect("SYMBOL won't be empty"));
    }

    let charset_len = charset.len();
    for _ in 0..(length - password.len() as u8) {
        let idx = rng.gen_range(0..charset_len);
        password.push(charset[idx]);
    }

    password.shuffle(&mut rng);
    let password = String::from_utf8_lossy(password.as_slice()).into_owned();

    println!("{}", password);

    let estimate = zxcvbn(&password, &[])?;

    eprintln!("Estimated strength: {}\n", estimate.score());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_genpass() {
        process_genpass(8, true, true, true, true).unwrap();
        process_genpass(16, true, true, true, true).unwrap();
        process_genpass(16, true, true, true, false).unwrap();
    }
}
