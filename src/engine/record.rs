//! Record handler. We choose to favor speed and efficient memory consumtion
//! by using an unsafe parsing mechanism for numerical value from bytes.
//! But it should be fine because we are protected by the type system.

use super::protocol::TransactionKind;
use csv::ByteRecord;

// /// This implementation would work with Serde with 'zero allocation'
// /// but we choose to favor speed in this use case.
// use serde::Deserialize;
// #[derive(Debug, Deserialize)]
// pub struct RecordRaw<'a> {
//     transaction: &'a [u8],
//     client: &'a [u8],
//     tx: &'a [u8],
//     amount: &'a [u8],
// }

#[derive(Debug, PartialEq)]
pub struct Record {
    pub transaction_kind: TransactionKind,
    pub client: u16,
    pub tx: u32,
    // This field is not protected by the type system and so needs
    // validation during parsing because we guess a negative number
    // would be odd here
    pub amount: f32,
}

impl Record {
    /// Returns a [`Record`] from a [`csv::ByteRecord`].
    pub fn from_byterecord(record: &mut ByteRecord) -> Result<Self, RecordError> {
        record.trim();
        if let (Some(txk), Some(client), Some(tx), Some(amount)) = (
            TransactionKind::new(&record[0]),
            parse_unchecked(&record[1]),
            parse_unchecked(&record[2]),
            parse_unchecked_f32(&record[3]),
        ) {
            // Round to 4 places past the decimal if we are not sure
            // about the input source.
            // let amount = (amount * 1_000.0).trunc() / 1_000.0;

            let record = Self {
                transaction_kind: txk,
                client,
                tx,
                amount,
            };
            if record.is_valid() {
                Ok(record)
            } else {
                Err(RecordError::Invalid)
            }
        } else {
            Err(RecordError::Parse)
        }
    }

    /// Checks the validity of this [`Record`].
    /// Only checks if the amount is positive at the moment.
    pub fn is_valid(&self) -> bool {
        self.amount >= 0.0
    }
}

/// For performance reason we use an unsafe unchecked bytes parsing
/// because we discard bad shaped record by leveraging protection from
/// the type system.
pub fn parse_unchecked<T: Sized + std::str::FromStr>(x: &[u8]) -> Option<T> {
    if let Ok(y) = unsafe { std::str::from_utf8_unchecked(x).parse::<T>() } {
        Some(y)
    } else {
        None
    }
}

/// Uses an unsafe parsing for performance reason. Implemented as a separate
/// function in order to default to float value : 0.0.
pub fn parse_unchecked_f32(x: &[u8]) -> Option<f32> {
    let v = unsafe {
        std::str::from_utf8_unchecked(x)
            .parse::<f32>()
            .unwrap_or_default()
    };

    if v == std::f32::NEG_INFINITY {
        None
    } else if v == std::f32::INFINITY {
        None
    } else {
        Some(v)
    }
}

/// Record Error implementation.
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum RecordError {
    /// Error occuring while parsing values from
    /// byte records.
    Parse,
    Invalid,
}

#[test]
fn test_record_parsing() {
    let csv_row = vec!["deposit", "    1", "3", "2.0"];
    let mut byte_record = ByteRecord::from(csv_row);

    let record = Record {
        transaction_kind: TransactionKind::Deposit,
        client: 1,
        tx: 3,
        amount: 2.0,
    };
    assert_eq!(record, Record::from_byterecord(&mut byte_record).unwrap());
}

#[test]
fn test_parsing_bad_record_transaction() {
    let csv_row = vec!["rule the world", "  xxx", "3", "2.0"];
    let mut byte_record = ByteRecord::from(csv_row);

    assert!(Record::from_byterecord(&mut byte_record).is_err());
}

#[test]
fn test_parsing_numerical_bad_record() {
    let csv_row = vec!["resolve", "  xxx", "3", "2.0"];
    let mut byte_record = ByteRecord::from(csv_row);

    assert!(Record::from_byterecord(&mut byte_record).is_err());
}

#[test]
fn test_record_is_valid() {
    let csv_row = vec!["deposit", "  7", "3", "-10.0"];
    let mut byte_record = ByteRecord::from(csv_row);

    assert!(Record::from_byterecord(&mut byte_record).is_err());
}

#[test]
fn test_record_disallow_neg_infinity() {
    // A -Inf parsing result should occur with value '-3.5e38'
    // and Inf with '3.5e38'.
    let csv_row = vec!["withdrawal", "  7", "3", "-3.5e38"];
    let mut byte_record = ByteRecord::from(csv_row);

    assert!(Record::from_byterecord(&mut byte_record).is_err());
}

#[test]
fn test_record_disallow_infinity() {
    // A Inf parsing result should occur with value '3.5e38'.
    let csv_row = vec!["deposit", "  7", "3", "3.5e38"];
    let mut byte_record = ByteRecord::from(csv_row);

    assert!(Record::from_byterecord(&mut byte_record).is_err());
}