use sha2::Digest;
use std::time::SystemTime;
use uuid::Uuid;

pub type PublicKey = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Transaction {
  Transfer {
    id: String,
    sender: PublicKey,
    receiver: PublicKey,
    amount: i64,
    timestamp: u128,
  },
}

impl Transaction {
  fn timestamp() -> u128 {
    SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_micros()
  }

  pub fn transfer(sender: PublicKey, receiver: PublicKey, amount: i64) -> Self {
    Transaction::Transfer {
      id: Uuid::new_v4().to_string(),
      sender,
      receiver,
      amount,
      timestamp: Transaction::timestamp(),
    }
  }

  pub fn hash(&self) -> String {
    let as_string = format!("{:?}", self);

    format!("{:x}", sha2::Sha256::digest(as_string.as_bytes()))
  }
}
