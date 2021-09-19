use sha2::Digest;
use std::time::SystemTime;
use uuid::Uuid;

pub type PublicKey = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Transaction {
  Transfer {
    id: Uuid,
    sender: PublicKey,
    receiver: PublicKey,
    amount: i64,
    timestamp: u128,
    signature: String,
  },
}

impl Transaction {
  pub fn transfer(sender: PublicKey, receiver: PublicKey, amount: i64) -> Self {
    Transaction::Transfer {
      id: Uuid::new_v4(),
      sender,
      receiver,
      amount,
      timestamp: SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_micros(),
      signature: String::from("TODO"),
    }
  }

  pub fn hash(&self) -> String {
    let as_string = format!("{:?}", self);

    format!("{:x}", sha2::Sha256::digest(as_string.as_bytes()))
  }
}
