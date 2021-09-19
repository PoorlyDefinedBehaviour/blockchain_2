use crate::transaction::PublicKey;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Account {
  accounts: HashSet<PublicKey>,
  balances: HashMap<PublicKey, i64>,
}

#[derive(Debug, PartialEq)]
pub enum AccountError {
  AccountNotFound(PublicKey),
}

impl Account {
  pub fn new() -> Self {
    Self {
      accounts: HashSet::new(),
      balances: HashMap::new(),
    }
  }

  pub fn add_account(&mut self, account: PublicKey) {
    if self.accounts.insert(account.clone()) {
      self.balances.insert(account, 0);
    }
  }

  pub fn balance(&self, account: &String) -> Option<i64> {
    self.balances.get(account).cloned()
  }

  // TODO: this is not atomic, will this be a problem?
  pub fn update_balance(&mut self, account: &String, amount: i64) -> Result<(), AccountError> {
    match self.balances.get(account) {
      None => Err(AccountError::AccountNotFound(account.clone())),
      Some(balance) => {
        // TODO: does it make sense for it to be possible to have
        // a negative balance?
        // if not, should we create a type to represent the balance
        // and enforce invariants?
        self.balances.insert(account.clone(), balance + amount);
        Ok(())
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn balance_returns_none_if_account_is_not_known() {
    let account = Account::new();

    assert_eq!(None, account.balance(&String::from("public_key")));
  }

  #[test]
  fn adds_new_public_key() {
    let mut account = Account::new();

    let public_key = String::from("public_key");

    account.add_account(public_key.clone());

    assert_eq!(account.accounts.contains(&public_key), true);
  }

  #[test]
  fn when_public_key_is_added_its_balance_is_zero() {
    let mut account = Account::new();

    let public_key = String::from("public_key");

    account.add_account(public_key.clone());

    assert_eq!(account.balance(&public_key), Some(0));
  }

  #[test]
  fn if_public_key_is_added_twice_it_wont_be_duplicated_and_its_balance_wont_be_altered() {
    let mut account = Account::new();

    let public_key = String::from("public_key");

    account.add_account(public_key.clone());

    account.update_balance(&public_key, 10).unwrap();

    assert_eq!(account.balance(&public_key), Some(10));
    assert_eq!(account.accounts.len(), 1)
  }

  #[test]
  fn if_we_try_to_update_the_balance_of_an_unknown_account_an_error_is_returned() {
    let mut account = Account::new();

    let public_key = String::from("unknown_public_key");

    let expected = Err(AccountError::AccountNotFound(public_key.clone()));

    let actual = account.update_balance(&public_key, 10);

    assert_eq!(expected, actual);
  }

  #[test]
  fn updates_account_balance() {
    let mut account = Account::new();

    let public_key = String::from("public_key");

    account.add_account(public_key.clone());

    assert_eq!(account.balance(&public_key), Some(0));

    account.update_balance(&public_key, 5).unwrap();

    assert_eq!(account.balance(&public_key), Some(5));

    account.update_balance(&public_key, -3).unwrap();

    assert_eq!(account.balance(&public_key), Some(2));
  }
}
