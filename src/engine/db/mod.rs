//! Database handling part where are stored the Client account's states and the important
//! transactions that we need to keep track of.
pub mod client;
mod error;
use client::ClientDB;
use crate::engine::protocol::Transaction;
pub use error::{DBError, Result};
use std::collections::BTreeMap;

pub type TransactionDB = BTreeMap<u32, Transaction>;

/// Data struct used to store our databases. Uses BTreeMaps under the hood for speed.
pub struct DB {
    client_db: ClientDB,
    transaction_db: TransactionDB,
}

impl DB {
    pub fn new() -> Self {
        Self {
            client_db: ClientDB::new(),
            transaction_db: TransactionDB::new(),
        }
    }

    pub fn get_client_db(&self) -> &ClientDB {
        &self.client_db
    }

    pub fn get_mut_client_db(&mut self) -> &mut ClientDB {
        &mut self.client_db
    }

    #[allow(dead_code)]
    pub fn get_transaction_db(&self) -> &TransactionDB {
        &self.transaction_db
    }

    pub fn get_mut_transaction_db(&mut self) -> &mut TransactionDB {
        &mut self.transaction_db
    }
}
