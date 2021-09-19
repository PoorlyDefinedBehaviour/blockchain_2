use crate::block::Block;
use crate::transaction::Transaction;
use hex;
use openssl::{
  hash::MessageDigest,
  pkey::{PKey, Private},
  rsa::Rsa,
  sign::{Signer, Verifier},
};

#[derive(Debug)]
pub struct Wallet {
  key_pair: Rsa<Private>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SignedTransaction {
  signature: String,
  transaction: Transaction,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SignedBlock {
  signature: String,
  block: Block,
}

impl Wallet {
  pub fn new() -> Self {
    Wallet {
      key_pair: Rsa::generate(2048).unwrap(),
    }
  }

  pub fn sign_transaction(&self, transaction: Transaction) -> SignedTransaction {
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

  pub fn verify_transaction(
    &self,
    SignedTransaction {
      signature,
      transaction,
    }: &SignedTransaction,
  ) -> bool {
    let key_pair = PKey::from_rsa(self.key_pair.clone()).unwrap();

    let mut verifier = Verifier::new(MessageDigest::sha256(), &key_pair).unwrap();

    verifier.update(transaction.hash().as_bytes()).unwrap();

    verifier.verify(&hex::decode(signature).unwrap()).unwrap()
  }

  pub fn sign_block(&self, block: Block) -> SignedBlock {
    let key_pair = PKey::from_rsa(self.key_pair.clone()).unwrap();

    let mut signer = Signer::new(MessageDigest::sha256(), &key_pair).unwrap();

    signer.update(block.hash().as_bytes()).unwrap();

    let mut buffer = vec![0; signer.len().unwrap()];

    signer.sign(&mut buffer).unwrap();

    SignedBlock {
      signature: hex::encode(buffer),
      block,
    }
  }

  pub fn verify_block(&self, SignedBlock { signature, block }: &SignedBlock) -> bool {
    let key_pair = PKey::from_rsa(self.key_pair.clone()).unwrap();

    let mut verifier = Verifier::new(MessageDigest::sha256(), &key_pair).unwrap();

    verifier.update(block.hash().as_bytes()).unwrap();

    verifier.verify(&hex::decode(signature).unwrap()).unwrap()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn verifies_transactions_signed_by_same_wallet() {
    let wallet = Wallet::new();

    let transaction = Transaction::transfer(
      String::from("sender_public_key"),
      String::from("receiver_public_key"),
      10,
    );

    let signed_transaction = wallet.sign_transaction(transaction.clone());

    assert_eq!(wallet.verify_transaction(&signed_transaction), true)
  }

  #[test]
  fn verifies_transactions_signed_by_other_wallets() {
    let wallet_a = Wallet::new();

    let transaction = Transaction::transfer(
      String::from("sender_public_key"),
      String::from("receiver_public_key"),
      10,
    );

    let transaction_signed_by_wallet_a = wallet_a.sign_transaction(transaction.clone());

    let wallet_b = Wallet::new();

    assert_eq!(
      wallet_b.verify_transaction(&transaction_signed_by_wallet_a),
      false
    )
  }

  #[test]
  fn verifies_blocks_signed_by_same_wallet() {
    let wallet = Wallet::new();

    let block = Block::new(
      Vec::new(),
      String::from("last_hash"),
      String::from("forger_public_key"),
      1,
    );

    let signed_block = wallet.sign_block(block);

    assert_eq!(wallet.verify_block(&signed_block), true)
  }

  #[test]
  fn verifies_blocks_signed_by_other_wallets() {
    let wallet_a = Wallet::new();

    let block = Block::new(
      Vec::new(),
      String::from("last_hash"),
      String::from("forger_public_key"),
      1,
    );

    let block_signed_by_wallet_a = wallet_a.sign_block(block);

    let wallet_b = Wallet::new();

    assert_eq!(wallet_b.verify_block(&block_signed_by_wallet_a), false)
  }
}
