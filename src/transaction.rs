use std::time::SystemTime;
use uuid::Uuid;

type PublicKey = String;

#[derive(Debug)]
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
}
