//! # Wallet
//!
//! This module exposes all the datatype related to jab wallets

mod errors;

pub use errors::WalletError;
use errors::WalletResult;

use secp256k1::{
    constants::SECRET_KEY_SIZE, ecdsa::Signature, rand::rngs::OsRng, Message, PublicKey, Secp256k1,
    SecretKey,
};
use std::str::FromStr;

/// Jab wallet type
pub struct Wallet {
    public_key: PublicKey,
    secret_key: SecretKey,
}

impl Wallet {
    /// Generate a new wallet
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        Self {
            public_key,
            secret_key,
        }
    }

    /// Get public key as bytes
    pub fn public_key(&self) -> String {
        self.public_key.to_string()
    }

    /// Get secret key
    pub fn secret_key(&self) -> [u8; SECRET_KEY_SIZE] {
        self.secret_key.secret_bytes()
    }

    /// Verify whether provided message has actually been signed with this key
    pub fn verify(&self, message: &[u8], signature: &str) -> WalletResult<bool> {
        let secp = Secp256k1::new();
        let message = Message::from_slice(message)?;
        let signature = Signature::from_str(signature)?;
        Ok(secp
            .verify_ecdsa(&message, &signature, &self.public_key)
            .is_ok())
    }

    /// Sign message
    pub fn sign(&self, message: &[u8]) -> WalletResult<String> {
        let secp = Secp256k1::new();
        let message = Message::from_slice(message)?;
        Ok(secp.sign_ecdsa(&message, &self.secret_key).to_string())
    }
}

impl TryFrom<&[u8]> for Wallet {
    type Error = WalletError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let secret_key = SecretKey::from_slice(data).map_err(WalletError::from)?;
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        Ok(Self {
            public_key,
            secret_key,
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn should_generate_valid_wallet_keys() {
        let wallet = Wallet::new();
        let signature = wallet.sign(&[0xab; 32]).unwrap();
        assert_eq!(wallet.verify(&[0xab; 32], &signature).unwrap(), true);
    }

    #[test]
    fn should_fail_wallet_verify() {
        let wallet = Wallet::new();
        let signature = wallet.sign(&[0xab; 32]).unwrap();

        let other_wallet = Wallet::new();
        assert_eq!(other_wallet.verify(&[0xab; 32], &signature).unwrap(), false);
    }

    #[test]
    fn should_generate_wallet_from_keys() {
        let wallet = Wallet::new();
        let copy_wallet = Wallet::try_from(wallet.secret_key().as_slice()).unwrap();
        assert_eq!(copy_wallet.public_key(), wallet.public_key());
    }
}
