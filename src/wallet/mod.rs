//! # Wallet
//!
//! This module exposes all the datatype related to jab wallets

mod errors;

pub use errors::WalletError;
use errors::WalletResult;

use data_encoding::HEXLOWER;
use ring::digest::{Context, SHA256};
use ripemd::{Digest, Ripemd160};
pub use secp256k1::constants::SECRET_KEY_SIZE;
use secp256k1::{ecdsa::Signature, rand::rngs::OsRng, Message, PublicKey, Secp256k1, SecretKey};
use std::str::FromStr;

/// Jab wallet type
pub struct Wallet {
    /// The wallet address corresponds to RIPEMD160(SHA256(public_key))
    address: String,
    /// Wallet public key
    public_key: PublicKey,
    /// Wallet secret key (DON'T SHARE WITH ANYBODY)
    secret_key: SecretKey,
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

impl Wallet {
    /// Generate a new wallet
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        Self {
            address: Self::calc_address(&public_key),
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

    /// Return wallet address
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Verify whether provided message has actually been signed with this key
    pub fn verify(message: &[u8], signature: &str, pubkey: &str) -> WalletResult<bool> {
        let pubkey = PublicKey::from_str(pubkey)?;
        let secp = Secp256k1::new();
        let message = Message::from_slice(message)?;
        let signature = Signature::from_str(signature)?;
        Ok(secp.verify_ecdsa(&message, &signature, &pubkey).is_ok())
    }

    /// Sign message
    pub fn sign(&self, message: &[u8]) -> WalletResult<String> {
        let secp = Secp256k1::new();
        let message = Message::from_slice(message)?;
        Ok(secp.sign_ecdsa(&message, &self.secret_key).to_string())
    }

    /// Calculate the wallet address
    ///
    /// The address format is `jab{RIPEMD160(SHA256(pubkey))}`
    fn calc_address(pubkey: &PublicKey) -> String {
        let mut digest_ctx = Context::new(&SHA256);
        digest_ctx.update(pubkey.to_string().as_bytes());
        let sha256 = digest_ctx.finish();
        let mut ripe_hasher = Ripemd160::new();
        ripe_hasher.update(sha256);
        let result = ripe_hasher.finalize();
        format!("jab{}", HEXLOWER.encode(&result))
    }
}

impl TryFrom<&[u8]> for Wallet {
    type Error = WalletError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let secret_key = SecretKey::from_slice(data).map_err(WalletError::from)?;
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        Ok(Self {
            address: Self::calc_address(&public_key),
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
        assert_eq!(
            Wallet::verify(&[0xab; 32], &signature, &wallet.public_key()).unwrap(),
            true
        );
        assert!(wallet.address().starts_with("jab"));
    }

    #[test]
    fn should_fail_wallet_verify() {
        let wallet = Wallet::new();
        let signature = wallet.sign(&[0xab; 32]).unwrap();

        let other_wallet = Wallet::new();
        assert_eq!(
            Wallet::verify(&[0xab; 32], &signature, &other_wallet.public_key()).unwrap(),
            false
        );
    }

    #[test]
    fn should_generate_wallet_from_keys() {
        let wallet = Wallet::new();
        let copy_wallet = Wallet::try_from(wallet.secret_key().as_slice()).unwrap();
        assert_eq!(copy_wallet.public_key(), wallet.public_key());
    }
}
