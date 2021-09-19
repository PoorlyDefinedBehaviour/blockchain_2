use crate::wallet::SignedBlock;

#[derive(Debug)]
pub struct Chain {
  blocks: Vec<SignedBlock>,
}

impl Chain {
  pub fn new() -> Self {
    Self { blocks: Vec::new() }
  }

  pub fn add(&mut self, block: SignedBlock) {
    self.blocks.push(block);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::block::Block;
  use crate::wallet::Wallet;

  #[test]
  fn adds_blocks_to_the_chain() {
    let wallet = Wallet::new();

    let block = wallet.sign_block(Block::new(
      Vec::new(),
      String::from("last_hash"),
      String::from("forger_public_key"),
      1,
    ));

    let mut chain = Chain::new();

    chain.add(block.clone());

    assert_eq!(chain.blocks, vec![block]);
  }
}
