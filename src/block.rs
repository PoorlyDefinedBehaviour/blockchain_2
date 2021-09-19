use crate::transaction::PublicKey;
use crate::wallet::SignedTransaction;
use sha2::Digest;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
  transactions: Vec<SignedTransaction>,
  last_hash: String,
  forger: PublicKey,
  block_count: u128,
  timestamp: u128,
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
      timestamp: SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_micros(),
    }
  }

  pub fn hash(&self) -> String {
    let as_string = format!("{:?}", self);

    format!("{:x}", sha2::Sha256::digest(as_string.as_bytes()))
  }

  pub fn genesis() -> Block {
    let mut block = Block::new(
      Vec::new(),
      String::from("genesis_hash"),
      String::from("genesis_forger"),
      0,
    );

    block.timestamp = 0;

    block
  }

  pub fn last_hash(&self) -> String {
    self.last_hash.clone()
  }

  pub fn block_count(&self) -> u128 {
    self.block_count
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn genesis_block_timestamp_is_always_zero() {
    let expected = Block {
      transactions: Vec::new(),
      last_hash: String::from("genesis_hash"),
      forger: String::from("genesis_forger"),
      block_count: 0,
      timestamp: 0,
    };

    let block = Block::genesis();

    assert_eq!(expected, block);
  }

  #[test]
  fn returns_the_previous_block_hash() {
    let block = Block::genesis();

    assert_eq!(block.last_hash(), String::from("genesis_hash"));
  }

  #[test]
  fn returns_the_block_block_count() {
    let block = Block::genesis();

    assert_eq!(0, block.block_count());
  }
}
