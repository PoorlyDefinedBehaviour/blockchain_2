use crate::transaction::{PublicKey, Transaction};
use crate::wallet::SignedTransaction;
use serde::{Deserialize, Serialize};
use std::convert::Into;
#[derive(Serialize)]
pub struct Message {
  pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddTransactionInput {
  pub public_key: PublicKey,
  pub signature: String,
  pub id: String,
  pub sender: PublicKey,
  pub receiver: PublicKey,
  pub amount: i64,
  pub timestamp: u128,
}

impl Into<SignedTransaction> for AddTransactionInput {
  fn into(self) -> SignedTransaction {
    SignedTransaction {
      signature: self.signature,
      transaction: Transaction::Transfer {
        id: self.id,
        sender: self.sender,
        receiver: self.receiver,
        amount: self.amount,
        timestamp: self.timestamp,
      },
    }
  }
}
