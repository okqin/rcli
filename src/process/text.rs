use crate::process_genpass;
use anyhow::{anyhow, Result};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use std::io::Read;

pub trait TextSigner {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextVerifier {
    fn verify(&self, reader: &mut dyn Read, signature: &[u8]) -> Result<bool>;
}

pub trait TextEncryptor {
    fn encrypt(&self, nonce: &[u8], plaintext: &[u8]) -> Result<Vec<u8>>;
}

pub trait TextDecrypter {
    fn decrypt(&self, nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>>;
}

pub struct Blake3 {
    key: [u8; 32],
}

pub struct Ed25519Signer {
    key: SigningKey,
}

pub struct Ed25519Verifier {
    key: VerifyingKey,
}

pub struct MyChaCha20Poly1305(ChaCha20Poly1305);

impl TextSigner for Blake3 {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(blake3::keyed_hash(&self.key, &buf).as_bytes().to_vec())
    }
}

impl TextVerifier for Blake3 {
    fn verify(&self, reader: &mut dyn Read, signature: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let expected = blake3::keyed_hash(&self.key, &buf);
        Ok(expected.as_bytes() == signature)
    }
}

impl TextSigner for Ed25519Signer {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let signature = self.key.sign(&buf);
        Ok(signature.to_bytes().to_vec())
    }
}

impl TextVerifier for Ed25519Verifier {
    fn verify(&self, reader: &mut dyn Read, signature: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let signature = Signature::try_from(signature)?;
        Ok(self.key.verify(&buf, &signature).is_ok())
    }
}

impl TextEncryptor for MyChaCha20Poly1305 {
    fn encrypt(&self, nonce: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
        let nonce = Nonce::from_slice(nonce);
        match self.0.encrypt(nonce, plaintext) {
            Ok(ciphertext) => Ok(ciphertext),
            Err(e) => Err(anyhow!("encryption failed: {}", e)),
        }
    }
}

impl TextDecrypter for MyChaCha20Poly1305 {
    fn decrypt(&self, nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
        let nonce = Nonce::from_slice(nonce);
        match self.0.decrypt(nonce, ciphertext) {
            Ok(plaintext) => Ok(plaintext),
            Err(e) => Err(anyhow!("decryption failed: {}", e)),
        }
    }
}

impl Blake3 {
    fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        let key = key.try_into()?;
        let signing_key = Self::new(key);
        Ok(signing_key)
    }

    fn generate() -> Result<Vec<[u8; 32]>> {
        let key = process_genpass(32, true, true, true, true)?;
        match key.try_into() {
            Ok(key) => Ok(vec![key]),
            Err(_) => Err(anyhow!("generate key length is not 32 bytes")),
        }
    }
}

impl Ed25519Signer {
    fn new(key: SigningKey) -> Self {
        Self { key }
    }

    fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        let key = key.try_into()?;
        let signing_key = Self::new(SigningKey::from_bytes(key));
        Ok(signing_key)
    }

    fn generate() -> Result<Vec<[u8; 32]>> {
        let mut csprng = OsRng;
        let key = SigningKey::generate(&mut csprng);
        let sk = key.to_bytes();
        let pk = key.verifying_key().to_bytes();
        Ok(vec![sk, pk])
    }
}

impl Ed25519Verifier {
    fn new(key: VerifyingKey) -> Self {
        Self { key }
    }

    fn try_new(key: impl AsRef<[u8]>) -> Result<Self> {
        let key = key.as_ref();
        let key = key.try_into()?;
        let verifying_key = Self::new(VerifyingKey::from_bytes(key)?);
        Ok(verifying_key)
    }
}

impl MyChaCha20Poly1305 {
    fn new(key: &[u8]) -> Self {
        let key = Key::from_slice(key);
        Self(ChaCha20Poly1305::new(key))
    }
}

pub fn process_text_sign(message: &mut dyn Read, key: &[u8], format: &str) -> Result<Vec<u8>> {
    let signature = match format {
        "blake3" => {
            let blake3 = Blake3::try_new(key)?;
            blake3.sign(message)?
        }
        "ed25519" => {
            let ed25519 = Ed25519Signer::try_new(key)?;
            ed25519.sign(message)?
        }
        _ => return Err(anyhow::anyhow!("unsupported format: {}", format)),
    };
    Ok(signature)
}

pub fn process_text_verify(
    message: &mut dyn Read,
    key: &[u8],
    format: &str,
    signature: &[u8],
) -> Result<bool> {
    let result = match format {
        "blake3" => {
            let blake3 = Blake3::try_new(key)?;
            blake3.verify(message, signature)?
        }
        "ed25519" => {
            let ed25519 = Ed25519Verifier::try_new(key)?;
            ed25519.verify(message, signature)?
        }
        _ => return Err(anyhow::anyhow!("unsupported format: {}", format)),
    };
    Ok(result)
}

pub fn process_text_generate_key(format: &str) -> Result<Vec<[u8; 32]>> {
    match format {
        "blake3" => Blake3::generate(),
        "ed25519" => Ed25519Signer::generate(),
        _ => Err(anyhow::anyhow!("unsupported format: {}", format)),
    }
}

pub fn process_text_encrypt(message: &[u8], key: &[u8], format: &str) -> Result<Vec<u8>> {
    let encrypted = match format {
        "chacha20poly1305" => {
            let cipher = MyChaCha20Poly1305::new(key);
            let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
            let mut ciphertext = cipher.encrypt(&nonce, message.as_ref())?;
            ciphertext.extend_from_slice(&nonce);
            ciphertext
        }
        _ => return Err(anyhow::anyhow!("unsupported format: {}", format)),
    };
    Ok(encrypted)
}

pub fn process_text_decrypt(message: &[u8], key: &[u8], format: &str) -> Result<Vec<u8>> {
    let decrypted = match format {
        "chacha20poly1305" => {
            let (ciphertext, nonce) = message.split_at(message.len() - 12);
            let cipher = MyChaCha20Poly1305::new(key);
            cipher.decrypt(nonce, ciphertext)?
        }
        _ => return Err(anyhow::anyhow!("unsupported format: {}", format)),
    };
    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_sign_verify() {
        let key = process_text_generate_key("blake3").unwrap();
        let blake3 = Blake3::try_new(key[0]).unwrap();
        let message = b"hello world!";
        let signature = blake3.sign(&mut &message[..]).unwrap();
        assert!(blake3.verify(&mut &message[..], &signature).unwrap());
    }

    #[test]
    fn test_ed25519_sign_verify() {
        let key = process_text_generate_key("ed25519").unwrap();
        let sign_key = Ed25519Signer::try_new(key[0]).unwrap();
        let message = b"hello world";
        let signature = sign_key.sign(&mut &message[..]).unwrap();
        let verify_key = Ed25519Verifier::try_new(key[1]).unwrap();
        assert!(verify_key.verify(&mut &message[..], &signature).unwrap());
    }

    #[test]
    fn test_chacha20poly1305_encrypt_decrypt() {
        let message = b"hello world!";
        let key = process_genpass(32, true, true, true, true).unwrap();
        let encrypt = process_text_encrypt(message, &key, "chacha20poly1305").unwrap();
        let decrypt = process_text_decrypt(&encrypt, &key, "chacha20poly1305").unwrap();
        assert_eq!(message, decrypt.as_slice());
    }
}
