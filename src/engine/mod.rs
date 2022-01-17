//! Transaction engine.
//! Read a csv transaction file and act accordingly.

mod db;
mod error;
mod protocol;
mod record;
use self::error::EngineErrorKind;
use db::{client::ClientAccountState, DBError};
use error::{EngineError, Result};
use num_format::{Locale, ToFormattedString};
use protocol::{Transaction, TransactionKind};
use record::Record;

#[allow(unused_imports)]
use db::client::ClientDB;
#[allow(unused_imports)]
use db::TransactionDB;

/// The orchestrator of all this. Actually, the necessary metadatas
/// are hard coded here. We migh move to to a configuration file later.
pub struct Engine<'a> {
    // Client and Transaction database
    db: db::DB,
    record_headers: Vec<&'a str>,
    output_header: &'a str,
}

impl<'a> Engine<'a> {
    pub fn new() -> Self {
        Self {
            db: db::DB::new(),
            record_headers: vec!["type", "client", "tx", "amount"],
            output_header: "client, available, held, total, locked",
        }
    }

    /// Read the csv file to process each transactions.
    pub fn process(&mut self, path: &str) -> Result<()> {
        let mut rdr = csv::Reader::from_path(path)?;
        let mut byte_record = csv::ByteRecord::new();

        // Checks if we are fed the correct headers.
        if rdr.byte_headers()? != self.record_headers {
            return Err(EngineError::new(EngineErrorKind::InvalidHeaders));
        }

        // TODO: Remove this !
        let mut counter = 0;
        while rdr.read_byte_record(&mut byte_record)? {
            // If the parsing fail, we just simply discard this record.
            if let Ok(record) = Record::from_byterecord(&mut byte_record) {
                // Process the Record and update the DB accordingly.
                // self.process_record(&record)?;
                match self.process_record(&record) {
                    Ok(()) => continue,
                    // We should pop a log message here if we fail
                    // to process a Record.
                    Err(_) => (),
                }
            } else {
                // We should probably pop a log message if a Record
                // parsing fail here.
                #[allow(clippy::unused_unit)]
                ()
            }

            // TODO: Remove this !
            // println!("{:#?}", self.db.client_db());
            // println!("\n---------------------------------------------\n");
            counter += 1;
            // println!(">> {} ", counter.to_formatted_string(&Locale::en));
            if counter % 10_000 == 0 {
                print!("\r>> {} ", counter.to_formatted_string(&Locale::en));
            }
        }
        println!();

        Ok(())
    }

    /// Updates [`ClientDB`] and the [`TransactionDB`] databases. Currently only
    /// the transaction one is updated only on deposit, dispute and resolve.
    fn process_record(&mut self, record: &Record) -> Result<()> {
        self.update_client_db(record)?;
        self.update_transaction_db(record)?;
        Ok(())
    }

    /// Updates the [`TransactionDB`] database. Currently only
    /// the deposit transaction kind is kept.
    pub fn update_transaction_db(&mut self, record: &Record) -> Result<()> {
        let trx_id = record.tx;

        // Only Deposit transactions are stored at the moment.
        if record.transaction_kind == TransactionKind::Deposit {
            match self.db.get_mut_transaction_db().get_mut(&trx_id) {
                Some(_) => return Err((DBError::TransactionAlreadyExists).into()),
                None => {
                    let trx = Transaction::from_record(record);
                    self.db.get_mut_transaction_db().insert(trx_id, trx);
                }
            }
        }
        Ok(())
    }

    /// Updates the [`ClientDB`] database accordingly from a [`Record`].
    pub fn update_client_db(&mut self, record: &Record) -> Result<()> {
        let key = record.client;

        match record.transaction_kind {
            TransactionKind::Deposit => {
                // If the client doesn't exist in the DB, we create it.
                self.db
                    .get_mut_client_db()
                    .entry(key)
                    .or_insert_with(ClientAccountState::new);

                if let Some(cas) = self.db.get_mut_client_db().get_mut(&key) {
                    cas.add(record.amount)?;
                }
            }
            TransactionKind::Withdrawal => {
                if let Some(cas) = self.db.get_mut_client_db().get_mut(&key) {
                    if cas.available() >= record.amount {
                        cas.sub(record.amount)?;
                    } else {
                        return Err(DBError::NotEnoughAvailableCredit.into());
                    }
                }
            }
            TransactionKind::Dispute => {
                // Lock the account until conflict resolution.
                if let Some(trx) = self.db.get_mut_transaction_db().get_mut(&record.tx) {
                    // Set a transaction as in dispute.
                    trx.set_dispute(true);

                    let amount = trx.amount();

                    // We should check that a dispute transaction's client_id refer
                    // to the same client_id from the original transaction
                    if trx.client_id() != record.client {
                        return Err(DBError::ClientIdMismatch.into());
                    }

                    if let Some(cas) = self.db.get_mut_client_db().get_mut(&key) {
                        cas.hold(amount);
                    } else {
                        // This Dispute transaction refer to an unknown Client,
                        // so we discard it and we return an error.
                        return Err(DBError::ClientNotFound.into());
                    }
                } else {
                    return Err(DBError::TransactionNotFound.into());
                }
            }
            TransactionKind::Resolve => {
                // Resolves a disputed transaction and release the held funds.
                if let Some(trx) = self.db.get_mut_transaction_db().get_mut(&record.tx) {
                    // This transaction is no longer in dispute.
                    trx.set_dispute(false);

                    let amount = trx.amount();

                    if let Some(cas) = self.db.get_mut_client_db().get_mut(&key) {
                        cas.unhold(amount)?;
                    } else {
                        return Err(DBError::ClientNotFound.into());
                    }
                } else {
                    return Err(DBError::TransactionNotFound.into());
                }
            }
            TransactionKind::Chargeback => {
                if let Some(trx) = self.db.get_mut_transaction_db().get_mut(&record.tx) {
                    let amount = trx.amount();

                    if trx.is_in_dispute() {
                        if let Some(cas) = self.db.get_mut_client_db().get_mut(&key) {
                            cas.unhold(amount)?;
                            cas.sub(amount)?;
                            cas.lock();
                        } else {
                            return Err(DBError::ClientNotFound.into());
                        }
                    } else {
                        return Err(DBError::TransactionNotInDispute.into());
                    }
                }
            }
            #[allow(unreachable_patterns)]
            _ => return Err(EngineError::new(EngineErrorKind::UnknownTransaction)),
        }

        Ok(())
    }

    /// Print the state of the client's account database.
    pub fn print_db(&self) {
        println!("{}", self.output_header);
        for (key, value) in self.db.get_client_db().iter() {
            let s = format!(
                "{:>6}, {:>9.4}, {:>4.4}, {:>5.4}, {:>6}",
                key,
                value.available(),
                value.held(),
                value.total(),
                value.locked()
            );
            println!("{}", s);
        }
    }
}

#[cfg(test)]
mod test_engine {
    use crate::engine::protocol;

    use super::*;
    use crate::engine::db::client;

    fn mock_engine<'a>() -> Engine<'a> {
        let mut engine = Engine::new();

        let mut cas1 = client::ClientAccountState::new();
        let mut cas2 = client::ClientAccountState::new();
        cas1.add(10.0).unwrap();
        cas2.add(20.0).unwrap();

        let deposit_tk = protocol::TransactionKind::Deposit;
        let trx1 = protocol::Transaction::new(deposit_tk, 1, 10.0);
        let trx2 = protocol::Transaction::new(deposit_tk, 2, 20.0);

        engine.db.get_mut_client_db().insert(1, cas1);
        engine.db.get_mut_client_db().insert(2, cas2);

        engine.db.get_mut_transaction_db().insert(1, trx1);
        engine.db.get_mut_transaction_db().insert(2, trx2);

        engine
    }

    #[test]
    fn test_deposit() {
        let mut engine = mock_engine();

        let record = Record {
            transaction_kind: TransactionKind::Deposit,
            client: 1,
            tx: 3,
            amount: 10.0,
        };

        engine.process_record(&record).unwrap();
        assert_eq!(engine.db.get_client_db().get(&1).unwrap().total(), 20.0);
        // Checks if we keep tracks of the transaction because it's a deposit.
        assert_eq!(
            engine.db.get_transaction_db().get(&3).unwrap().amount(),
            10.0
        );
    }

    #[test]
    fn test_withdrawal() {
        let mut engine = mock_engine();

        let record = Record {
            transaction_kind: TransactionKind::Withdrawal,
            client: 1,
            tx: 3,
            amount: 3.0,
        };

        engine.process_record(&record).unwrap();
        assert_eq!(engine.db.get_client_db().get(&1).unwrap().total(), 7.0);
    }

    #[test]
    fn test_dispute() {
        let mut engine = mock_engine();

        let record_deposit = Record {
            transaction_kind: TransactionKind::Deposit,
            client: 1,
            tx: 3,
            amount: 3.0,
        };
        engine.process_record(&record_deposit).unwrap();

        let record_dispute = Record {
            transaction_kind: TransactionKind::Dispute,
            client: 1,
            tx: 3,
            amount: 0.0,
        };
        engine.process_record(&record_dispute).unwrap();

        let cas = engine.db.get_client_db().get(&1).unwrap();
        assert_eq!(cas.total(), 13.0);
        assert_eq!(cas.available(), 10.0);
        assert_eq!(cas.held(), 3.0);

        let tx = engine.db.get_transaction_db().get(&3).unwrap();
        assert_eq!(tx.is_in_dispute(), true);
    }

    #[test]
    fn test_resolve() {
        let mut engine = mock_engine();

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

        for record in records {
            engine.process_record(&record).unwrap();
        }

        let cas = engine.db.get_client_db().get(&1).unwrap();

        assert_eq!(cas.held(), 0.0);
        assert_eq!(cas.available(), 13.0);
        assert_eq!(cas.total(), 13.0);

        let tx = engine.db.get_transaction_db().get(&3).unwrap();
        assert_eq!(tx.is_in_dispute(), false);
    }

    #[test]
    fn test_chargeback() {
        let mut engine = mock_engine();

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
                transaction_kind: TransactionKind::Chargeback,
                client: 1,
                tx: 3,
                amount: 0.0,
            },
        ];

        for record in records {
            engine.process_record(&record).unwrap();
        }

        let cas = engine.db.get_client_db().get(&1).unwrap();

        assert_eq!(cas.locked(), true);
        assert_eq!(cas.held(), 0.0);
        assert_eq!(cas.available(), 10.0);
        assert_eq!(cas.total(), 10.0);

        let tx = engine.db.get_transaction_db().get(&3).unwrap();
        assert_eq!(tx.is_in_dispute(), true);
    }
}
