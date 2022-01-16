//! Database handling part.
mod client;
mod error;
// use super::error::EngineErrorKind;   
use super::protocol::{Transaction, TransactionDB, TransactionKind};
use super::record::Record;
use client::{ClientAccountState, ClientDB};
pub use error::{DBError, Result};

/// Wrapper around BTreeMap in order to init a dumb database.
#[derive(Debug)]
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

    pub fn client_db(&self) -> &ClientDB {
        &self.client_db
    }

    #[allow(dead_code)]
    pub fn transaction_db(&self) -> &TransactionDB {
        &self.transaction_db
    }

    /// Updates [`CLientDB`] and the [`TransactionDB`] databases. Currently only
    /// the transaction one is updated only on deposit, dispute and resolve.
    pub fn update(&mut self, record: &Record) -> Result<()> {
        self.update_client_db(record)?;
        self.update_transaction_db(record)?;
        Ok(())
    }

    pub fn update_client_db(&mut self, record: &Record) -> Result<()> {
        let key = record.client;

        // If the client doesn't exists in the DB, we create it.
        self.client_db
            .entry(key)
            .or_insert_with(ClientAccountState::new);
        // dbg!(&self.client_db());
        // dbg!(&record.transaction_kind);

        match record.transaction_kind {
            TransactionKind::Deposit => {
                if let Some(cas) = self.client_db.get_mut(&key) {
                    cas.add(record.amount)?;
                }
            },
            TransactionKind::Withdrawal => {
                if let Some(cas) = self.client_db.get_mut(&key) {
                    if cas.available() >= record.amount {
                        cas.sub(record.amount)?;
                    } else {
                        return Err(DBError::NotEnoughAvailableCredit);
                    }
                }
            },
            TransactionKind::Dispute => {
                // Lock the account until conflict resolution.
                if let Some(trx) = self.transaction_db.get(&record.tx) {
                    let amount = trx.amount();

                    // We should check that a dispute transaction's client_id refer
                    // to the same client_id from the original transaction
                    if trx.client_id() != record.client {
                        return Err(DBError::ClientIdMismatch);
                    }

                    if let Some(cas) = self.client_db.get_mut(&key) {
                        cas.hold(amount)?;
                    } else {
                        // This Dispute transaction refer to an unknown Client,
                        // so we discard it and we return an error.
                        return Err(DBError::ClientNotFound);
                    }
                } else {
                    return Err(DBError::TransactionNotFound);
                }
            },
            TransactionKind::Resolve => {
                // Resolves a disputed transaction and release the held funds.
                if let Some(trx) = self.transaction_db.get(&record.tx) {
                    let amount = trx.amount();

                    if let Some(cas) = self.client_db.get_mut(&key) {
                        cas.unhold(amount)?
                    }
                } else {
                    return Err(DBError::TransactionNotFound);
                }
            },
            TransactionKind::Chargeback => {
                // This is where we lock a Client Account
                if let Some(cas) = self.client_db.get_mut(&key) {
                    cas.lock()?;
                } else {
                    return Err(DBError::ClientNotFound);
                }
            },
            #[allow(unreachable_patterns)]
            _ => return Err(DBError::InvalidTransaction),
        }

        Ok(())
    }

    /// Updates the [`TransactionDB`] database. Currently only
    /// the deposit transaction kind is kept.
    pub fn update_transaction_db(&mut self, record: &Record) -> Result<()> {
        let trx_id = record.tx;

        match record.transaction_kind {
            TransactionKind::Deposit => match self.transaction_db.get_mut(&trx_id) {
                Some(_) => return Err(DBError::TransactionAlreadyExists),
                None => {
                    let trx = Transaction::from_record(&record);
                    self.transaction_db.insert(trx_id, trx);
                }
            },
            // TransactionKind::Dispute => {
            //     ()
            // },
            // TransactionKind::Resolve => {

            //     ()
            // }
            _ => (),
        }
        Ok(())
    }

    /// Checks if an operation is valid.
    pub fn is_valid(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod test_db {
    use crate::engine::protocol;

    use super::*;

    fn mock_db() -> DB {
        let mut db = DB::new();

        let mut cas1 = client::ClientAccountState::new();
        let mut cas2 = client::ClientAccountState::new();
        cas1.add(10.0).unwrap();
        cas2.add(20.0).unwrap();

        let deposit_tk = protocol::TransactionKind::Deposit;
        let trx1 = protocol::Transaction::new(deposit_tk, 1, 10.0);
        let trx2 = protocol::Transaction::new(deposit_tk, 2, 20.0);

        db.client_db.insert(1, cas1);
        db.client_db.insert(2, cas2);

        db.transaction_db.insert(1, trx1);
        db.transaction_db.insert(2, trx2);

        db
    }

    #[test]
    fn test_deposit() {
        let mut db = mock_db();

        let record = Record {
            transaction_kind: TransactionKind::Deposit,
            client: 1,
            tx: 3,
            amount: 10.0,
        };

        db.update(&record).unwrap();
        assert_eq!(db.client_db().get(&1).unwrap().total(), 20.0);
        // Checks if we kkep track of the transaction because it's a deposit.
        assert_eq!(db.transaction_db().get(&3).unwrap().amount(), 10.0);
    }

    #[test]
    fn test_withdrawal() {
        let mut db = mock_db();

        let record = Record {
            transaction_kind: TransactionKind::Withdrawal,
            client: 1,
            tx: 3,
            amount: 3.0,
        };

        db.update(&record).unwrap();
        assert_eq!(db.client_db().get(&1).unwrap().total(), 7.0);
    }

    #[test]
    fn test_dispute() {
        let mut db = mock_db();

        let record_deposit = Record {
            transaction_kind: TransactionKind::Deposit,
            client: 1,
            tx: 3,
            amount: 3.0,
        };
        db.update(&record_deposit).unwrap();

        let record_dispute = Record {
            transaction_kind: TransactionKind::Dispute,
            client: 1,
            tx: 3,
            amount: 0.0,
        };
        db.update(&record_dispute).unwrap();

        let cas = db.client_db().get(&1).unwrap();
        assert_eq!(cas.total(), 13.0);
        assert_eq!(cas.available(), 10.0);
        assert_eq!(cas.held(), 3.0);
    }

    #[test]
    fn test_resolve() {
        let mut db = mock_db();

        let records = [
            Record {
                transaction_kind: TransactionKind::Deposit,
                client: 1,
                tx: 3,
                amount: 3.0,
            },
            Record {
                transaction_kind: TransactionKind::Dispute,
                client: 1,
                tx: 3,
                amount: 0.0,
            },
            Record {
                transaction_kind: TransactionKind::Resolve,
                client: 1,
                tx: 3,
                amount: 0.0,
            },
        ];

        let _: Vec<Result<_>> = records
            .into_iter()
            .map(|record| db.update(&record))
            .collect();

        assert_eq!(db.client_db().get(&1).unwrap().held(), 0.0);
        assert_eq!(db.client_db().get(&1).unwrap().available(), 13.0);
        assert_eq!(db.client_db().get(&1).unwrap().total(), 13.0);
    }

    #[test]
    fn test_chargeback() {
        let mut db = mock_db();

        let record = Record {
            transaction_kind: TransactionKind::Withdrawal,
            client: 1,
            tx: 3,
            amount: 2.0,
        };

        db.update(&record).unwrap();
        assert_eq!(db.client_db().get(&1).unwrap().locked(), true);
        unimplemented!();
    }
}
