use crate::chain::Chain;
use crate::transaction_pool::TransactionPool;
use crate::wallet::Wallet;

#[derive(Debug)]
pub struct Node {
  transaction_pool: TransactionPool,
  wallet: Wallet,
  chain: Chain,
}

impl Node {
  pub fn new() -> Self {
    Self {
      transaction_pool: TransactionPool::new(),
      wallet: Wallet::new(),
      chain: Chain::new(),
    }
  }
}
