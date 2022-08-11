//! # Merkle
//!
//! This module expose the merkle tree used by the jab blockchain

use super::Transaction;

use merkle::MerkleTree;
use ring::digest::{Algorithm, SHA512};

static DIGEST_ALGO: &'static Algorithm = &SHA512;

pub struct JabMerkleTree {
    tree: MerkleTree<Transaction>,
}

impl JabMerkleTree {
    /// Create new Jab merkle tree
    pub fn new(transactions: Vec<Transaction>) -> Self {
        Self {
            tree: MerkleTree::from_vec(DIGEST_ALGO, transactions),
        }
    }

    /// Get root hash
    pub fn root_hash(&self) -> String {
        hex::encode(self.tree.root_hash())
    }
}
