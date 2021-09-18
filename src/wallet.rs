use crate::transaction::Transaction;
use hex;
use openssl::{
  hash::MessageDigest,
  pkey::{PKey, Private},
  rsa::Rsa,
  sign::Signer,
};

#[derive(Debug)]
pub struct Wallet {
  key_pair: Rsa<Private>,
}

#[derive(Debug)]
pub struct SignedTransaction {
  signature: String,
  transaction: Transaction,
}

impl Wallet {
  pub fn new() -> Self {
    Wallet {
      key_pair: Rsa::generate(2048).unwrap(),
    }
  }

  pub fn sign(&self, transaction: Transaction) -> SignedTransaction {
    let key_pair = PKey::from_rsa(self.key_pair.clone()).unwrap();

    let mut signer = Signer::new(MessageDigest::sha256(), &key_pair).unwrap();

    signer.update(transaction.hash().as_bytes()).unwrap();

    let mut buffer = vec![0; signer.len().unwrap()];

    signer.sign(&mut buffer).unwrap();

    SignedTransaction {
      signature: hex::encode(buffer),
      transaction,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use openssl::sign::Verifier;

  #[test]
  fn verifies_transactions_signed_by_same_wallet() {
    let wallet = Wallet::new();

    let transaction = Transaction::transfer(
      String::from("sender_public_key"),
      String::from("receiver_public_key"),
      10,
    );

    let signed_transaction = wallet.sign(transaction.clone());

    // TODO: this will be prettier if we make it a wallet method
    // Example: wallet.verify(transaction)
    let key_pair = PKey::from_rsa(wallet.key_pair.clone()).unwrap();

    let mut verifier = Verifier::new(MessageDigest::sha256(), &key_pair).unwrap();

    verifier.update(transaction.hash().as_bytes()).unwrap();

    assert_eq!(
      verifier
        .verify(&hex::decode(signed_transaction.signature).unwrap())
        .unwrap(),
      true
    );
  }

  #[test]
  fn verifies_transactions_signed_by_other_wallets() {
    let wallet_a = Wallet::new();

    let transaction = Transaction::transfer(
      String::from("sender_public_key"),
      String::from("receiver_public_key"),
      10,
    );

    let transaction_signed_by_wallet_a = wallet_a.sign(transaction.clone());

    let wallet_b = Wallet::new();

    let key_pair = PKey::from_rsa(wallet_b.key_pair.clone()).unwrap();

    let mut verifier = Verifier::new(MessageDigest::sha256(), &key_pair).unwrap();

    verifier.update(transaction.hash().as_bytes()).unwrap();

    assert_eq!(
      verifier
        .verify(&hex::decode(transaction_signed_by_wallet_a.signature).unwrap())
        .unwrap(),
      false
    );
  }
}
