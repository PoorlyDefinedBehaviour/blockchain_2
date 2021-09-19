use crate::account::Account;
use crate::wallet::SignedBlock;

#[derive(Debug)]
pub struct Chain {
  blocks: Vec<SignedBlock>,
  account: Account,
}

#[derive(Debug, PartialEq)]
pub enum ChainError {
  InvalidBlockHash(SignedBlock),
  InvalidBlockCount(SignedBlock),
}

impl Chain {
  pub fn new() -> Self {
    Self {
      blocks: vec![SignedBlock::genesis()],
      account: Account::new(),
    }
  }

  pub fn add(&mut self, block: SignedBlock) -> Result<(), ChainError> {
    let last_block = self.blocks.last().unwrap();

    // TODO: make hash a property of the block to avoid computing every time?
    // NOTE: is it safe to make the hash a property of the block?
    if last_block.hash() != block.last_hash() {
      return Err(ChainError::InvalidBlockHash(block));
    }

    if last_block.block_count() + 1 != block.block_count() {
      return Err(ChainError::InvalidBlockCount(block));
    }

    self.blocks.push(block);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::block::Block;
  use crate::wallet::Wallet;

  #[test]
  fn chain_starts_with_the_genesis_block() {
    let chain = Chain::new();

    assert_eq!(chain.blocks, vec![SignedBlock::genesis()]);
  }

  #[test]
  fn does_not_add_block_to_the_chain_if_last_block_hash_does_not_match() {
    let mut chain = Chain::new();

    let wallet = Wallet::new();

    let block = wallet.sign_block(Block::new(
      Vec::new(),
      String::from("not_the_last_block_hash"),
      String::from("forger_public_key"),
      0,
    ));

    let expected = Err(ChainError::InvalidBlockHash(block.clone()));

    let actual = chain.add(block);

    assert_eq!(expected, actual);
  }

  #[test]
  fn does_not_add_block_to_the_chain_if_block_count_does_not_match() {
    let mut chain = Chain::new();

    let wallet = Wallet::new();

    let block = wallet.sign_block(Block::new(
      Vec::new(),
      chain.blocks.last().unwrap().hash(),
      String::from("forger_public_key"),
      10,
    ));

    let expected = Err(ChainError::InvalidBlockCount(block.clone()));

    let actual = chain.add(block);

    assert_eq!(expected, actual);
  }

  #[test]
  fn adds_blocks_to_the_chain() {
    let mut chain = Chain::new();

    let wallet = Wallet::new();

    let block = wallet.sign_block(Block::new(
      Vec::new(),
      chain.blocks.last().unwrap().hash(),
      String::from("forger_public_key"),
      1,
    ));

    assert_eq!(Ok(()), chain.add(block.clone()));

    assert_eq!(chain.blocks, vec![SignedBlock::genesis(), block]);
  }
}
