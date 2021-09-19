use crate::transaction::PublicKey;
use crate::wallet::SignedTransaction;
use sha2::Digest;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
  transactions: Vec<SignedTransaction>,
  last_hash: String,
  forger: PublicKey,
  block_count: u128,
}

impl Block {
  pub fn new(
    transactions: Vec<SignedTransaction>,
    last_hash: String,
    forger: PublicKey,
    block_count: u128,
  ) -> Self {
    Self {
      transactions,
      last_hash,
      forger,
      block_count,
    }
  }

  pub fn hash(&self) -> String {
    let as_string = format!("{:?}", self);

    format!("{:x}", sha2::Sha256::digest(as_string.as_bytes()))
  }
}
