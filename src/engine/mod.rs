//! Transaction engine.
//! Read a csv transaction file and act accordingly.

mod db;
mod error;
mod protocol;
mod record;
use self::error::EngineErrorKind;
use error::{EngineError, Result};
use num_format::{Locale, ToFormattedString};
use record::Record;

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

        let mut counter = 0;
        while rdr.read_byte_record(&mut byte_record)? {
            // If the parsing fail, we just simply discard this record.
            if let Ok(record) = Record::from(&mut byte_record) {
                // println!("{:?}", &record);

                // Update the DB.
                match self.db.update(&record) {
                // match self.update(&record) {
                    Err(_e) => (),
                    _ => (),
                }
            } else {
                // We should probably pop a log message if a Record
                // parsing fail here.
                ()
            }

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


    // /// Updates [`CLientDB`] and the [`TransactionDB`] databases. Currently only
    // /// the transaction one is updated only on deposit, dispute and resolve.
    // pub fn update(&mut self, record: &Record) -> Result<()> {
    //     self.update_client_db(record)?;
    //     self.update_transaction_db(record)?;
    //     Ok(())
    // }

    /// Print the state of the client's account database.
    pub fn print_db(&self) {
        println!("{}", self.output_header);
        for (key, value) in self.db.client_db().iter() {
            let s = format!(
                "{:>6}, {:>9}, {:>4}, {:>5}, {:>6}",
                // "{:>6}, {:>9.4}, {:>4.4}, {:>5.4}, {:>6}",
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

// tests ideas :
// header testing
// X valid row
// X valid transactions
// X trim
//
