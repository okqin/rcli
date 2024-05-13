use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, decode_header, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Serialize};

pub fn process_jwt_sign_with_secret(
    payload: impl Serialize,
    key: &[u8],
    algorithm: &str,
) -> Result<String> {
    let key = &EncodingKey::from_secret(key);
    let alg = algorithm.parse()?;
    let header = Header::new(alg);
    encode(&header, &payload, key).map_err(|e| anyhow!("Failed to sign jwt: {e}"))
}

pub fn process_jwt_verify_with_secret<T: DeserializeOwned>(
    token: &str,
    key: &[u8],
    algorithm: Option<&str>,
) -> Result<T> {
    let key = &DecodingKey::from_secret(key);
    let alg = match algorithm {
        Some(alg) => alg.parse()?,
        None => {
            let header = decode_header(token)?;
            header.alg
        }
    };
    let mut validation = Validation::new(alg);
    validation.validate_aud = false;
    decode::<T>(token, key, &validation)
        .map(|data| data.claims)
        .map_err(|e| anyhow!("Failed to verify jwt: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestPayload {
        sub: String,
        aud: String,
        exp: u64,
    }

    #[test]
    fn test_jwt_sign_and_verify() {
        let payload = TestPayload {
            sub: "test".to_string(),
            aud: "test".to_string(),
            exp: Utc::now().timestamp() as u64,
        };
        let key = b"secret";
        let token = process_jwt_sign_with_secret(&payload, key, "HS256").unwrap();
        let data = process_jwt_verify_with_secret::<TestPayload>(&token, key, None).unwrap();
        assert_eq!(data, payload);
    }

    #[test]
    fn test_jwt_time_exp() {
        let payload = TestPayload {
            sub: "test".to_string(),
            aud: "test".to_string(),
            exp: (Utc::now().timestamp() - 61) as u64,
        };
        let key = b"secret";
        let token = process_jwt_sign_with_secret(payload, key, "HS256").unwrap();
        let data = process_jwt_verify_with_secret::<TestPayload>(&token, key, None);
        assert!(data.is_err());
    }
}
