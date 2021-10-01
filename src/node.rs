use crate::chain::Chain;
use crate::transaction::{PublicKey, Transaction};
use crate::wallet::{SignedTransaction, Wallet};
use std::collections::HashSet;

#[derive(Debug)]
pub struct Node {
  transactions: HashSet<SignedTransaction>,
  wallet: Wallet,
  chain: Chain,
}

#[derive(Debug, PartialEq)]
pub enum NodeError {
  InvalidSignature {
    public_key: PublicKey,
    signed_transaction: SignedTransaction,
  },
}

impl Node {
  pub fn new() -> Self {
    Self {
      transactions: HashSet::new(),
      wallet: Wallet::new(),
      chain: Chain::new(),
    }
  }

  pub fn transaction(
    &mut self,
    public_key: &PublicKey,
    transaction: SignedTransaction,
  ) -> Result<(), NodeError> {
    if !Wallet::verify_transaction(public_key, &transaction) {
      return Err(NodeError::InvalidSignature {
        public_key: public_key.clone(),
        signed_transaction: transaction.clone(),
      });
    }

    self.transactions.insert(transaction);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn returns_error_when_we_try_to_add_a_transaction_with_an_invalid_signature() {
    let wallet_a = Wallet::new();

    let transaction = Transaction::transfer(
      String::from("sender_public_key"),
      String::from("receiver_public_key"),
      10,
    );

    let transaction_signed_by_wallet_a = wallet_a.sign_transaction(transaction.clone());

    let wallet_b = Wallet::new();

    let mut node = Node::new();

    let expected = Err(NodeError::InvalidSignature {
      public_key: wallet_b.public_key(),
      signed_transaction: transaction_signed_by_wallet_a.clone(),
    });

    let actual = node.transaction(&wallet_b.public_key(), transaction_signed_by_wallet_a);

    assert_eq!(expected, actual);
  }

  #[test]
  fn adds_transaction_to_transaction_set() {
    let wallet = Wallet::new();

    let transaction = Transaction::transfer(
      String::from("sender_public_key"),
      String::from("receiver_public_key"),
      10,
    );

    let signed_transaction = wallet.sign_transaction(transaction.clone());

    let mut node = Node::new();

    let mut expected = HashSet::new();

    expected.insert(signed_transaction.clone());

    let actual = node.transaction(&wallet.public_key(), signed_transaction);

    assert_eq!(Ok(()), actual);

    assert_eq!(node.transactions, expected);
  }

  // NOTE: this is fine because Node::transaction
  // has no side effects at the moment
  #[test]
  fn each_transaction_is_only_added_once() {
    let wallet = Wallet::new();

    let transaction = Transaction::transfer(
      String::from("sender_public_key"),
      String::from("receiver_public_key"),
      10,
    );

    let signed_transaction = wallet.sign_transaction(transaction.clone());

    let mut node = Node::new();

    let mut expected = HashSet::new();

    expected.insert(signed_transaction.clone());

    node
      .transaction(&wallet.public_key(), signed_transaction.clone())
      .unwrap();
    node
      .transaction(&wallet.public_key(), signed_transaction)
      .unwrap();

    assert_eq!(node.transactions, expected);
  }
}
