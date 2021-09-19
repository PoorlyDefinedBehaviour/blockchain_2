use crate::wallet::SignedTransaction;
use std::collections::HashSet;

#[derive(Debug)]
pub struct TransactionPool {
  transactions: HashSet<SignedTransaction>,
}

impl TransactionPool {
  pub fn new() -> Self {
    Self {
      transactions: HashSet::new(),
    }
  }

  pub fn add(&mut self, transaction: SignedTransaction) {
    self.transactions.insert(transaction);
  }
}
