use crate::account::{Account, AccountError};
use crate::transaction::{PublicKey, Transaction};
use crate::wallet::{SignedBlock, SignedTransaction};

#[derive(Debug)]
pub struct Chain {
  blocks: Vec<SignedBlock>,
  account: Account,
}

#[derive(Debug, PartialEq)]
pub enum ChainError {
  InvalidBlockHash(SignedBlock),
  InvalidBlockCount(SignedBlock),
  AccountNotFound(PublicKey),
  TransactionsFailed(Vec<ChainError>),
}

impl Chain {
  pub fn new() -> Self {
    Self {
      blocks: vec![SignedBlock::genesis()],
      account: Account::new(),
    }
  }

  pub fn add(&mut self, signed_block: SignedBlock) -> Result<(), ChainError> {
    let last_block = self.blocks.last().unwrap();

    // TODO: make hash a property of the block to avoid computing it every time?
    // NOTE: is it safe to make the hash a property of the block?
    if last_block.hash() != signed_block.last_hash() {
      return Err(ChainError::InvalidBlockHash(signed_block));
    }

    if last_block.block_count() + 1 != signed_block.block_count() {
      return Err(ChainError::InvalidBlockCount(signed_block));
    }

    let mut failed_transactions = Vec::new();

    for transaction in &signed_block.block.transactions {
      if let Err(error) = self.execute(transaction) {
        failed_transactions.push(error);
      }
    }

    self.blocks.push(signed_block);

    if !failed_transactions.is_empty() {
      return Err(ChainError::TransactionsFailed(failed_transactions));
    }

    Ok(())
  }

  pub fn execute(
    &mut self,
    SignedTransaction { transaction, .. }: &SignedTransaction,
  ) -> Result<(), ChainError> {
    match transaction {
      Transaction::Transfer {
        sender,
        receiver,
        amount,
        ..
      } => {
        // TODO: not atomic, is this a problem?
        self
          .account
          .update_balance(&sender, -amount)
          .map_err(|AccountError::AccountNotFound(account)| ChainError::AccountNotFound(account))?;
        self
          .account
          .update_balance(&receiver, *amount)
          .map_err(|AccountError::AccountNotFound(account)| ChainError::AccountNotFound(account))
      }
    }
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
  fn after_adding_block_to_the_chain_returns_transactions_that_failed() {
    let mut chain = Chain::new();

    let sender = String::from("sender_public_key");

    let receiver = String::from("receiver_public_key");

    let wallet = Wallet::new();

    let transaction =
      wallet.sign_transaction(Transaction::transfer(sender.clone(), receiver.clone(), 10));

    let block = wallet.sign_block(Block::new(
      vec![transaction],
      chain.blocks.last().unwrap().hash(),
      String::from("forger_public_key"),
      1,
    ));

    let expected = Err(ChainError::TransactionsFailed(vec![
      ChainError::AccountNotFound(sender),
    ]));

    let actual = chain.add(block);

    assert_eq!(expected, actual);
  }

  #[test]
  fn block_is_added_to_the_chain_even_if_one_of_its_transactions_fail() {
    let mut chain = Chain::new();

    let sender = String::from("sender_public_key");

    let receiver = String::from("receiver_public_key");

    let wallet = Wallet::new();

    let transaction =
      wallet.sign_transaction(Transaction::transfer(sender.clone(), receiver.clone(), 10));

    let block = wallet.sign_block(Block::new(
      vec![transaction],
      chain.blocks.last().unwrap().hash(),
      String::from("forger_public_key"),
      1,
    ));

    assert_eq!(
      Err(ChainError::TransactionsFailed(vec![
        ChainError::AccountNotFound(sender)
      ])),
      chain.add(block.clone())
    );

    assert_eq!(chain.blocks, vec![SignedBlock::genesis(), block]);
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

  #[test]
  fn executes_block_transactions_before_adding_it_to_the_chain() {
    let mut chain = Chain::new();

    let sender = String::from("sender_public_key");

    chain.account.add_account(sender.clone());

    chain.account.update_balance(&sender, 10).unwrap();

    let receiver = String::from("receiver_public_key");

    chain.account.add_account(receiver.clone());

    let wallet = Wallet::new();

    let transaction =
      wallet.sign_transaction(Transaction::transfer(sender.clone(), receiver.clone(), 10));

    let block = wallet.sign_block(Block::new(
      vec![transaction],
      chain.blocks.last().unwrap().hash(),
      String::from("forger_public_key"),
      1,
    ));

    chain.add(block).unwrap();

    assert_eq!(chain.account.balance(&sender), Some(0));
    assert_eq!(chain.account.balance(&receiver), Some(10));
  }

  #[test]
  fn executes_transfer_transaction() {
    let mut chain = Chain::new();

    let sender = String::from("sender_public_key");

    chain.account.add_account(sender.clone());

    chain.account.update_balance(&sender, 10).unwrap();

    let receiver = String::from("receiver_public_key");

    chain.account.add_account(receiver.clone());

    let wallet = Wallet::new();

    let transaction =
      wallet.sign_transaction(Transaction::transfer(sender.clone(), receiver.clone(), 10));

    chain.execute(&transaction).unwrap();

    assert_eq!(chain.account.balance(&sender), Some(0));
    assert_eq!(chain.account.balance(&receiver), Some(10));
  }
}
