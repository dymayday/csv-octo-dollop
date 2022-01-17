//! Error handling part.

// use std::error::Error as StdError;
use super::record::RecordError;
use crate::engine::db::DBError;
use csv::Error as CsvError;
use std::fmt;

/// A type alias for `Result<T, engine::Error>`.
pub type Result<T> = std::result::Result<T, EngineError>;

#[derive(Debug)]
pub struct EngineError(Box<EngineErrorKind>);

impl EngineError {
    pub(crate) fn new(kind: EngineErrorKind) -> EngineError {
        EngineError(Box::new(kind))
    }

    // pub fn kind(&self) -> &EngineErrorKind {
    //     &self.0
    // }

    // pub fn into_kind(self) -> EngineErrorKind {
    //     *self.0
    // }
}
#[derive(Debug)]
pub enum EngineErrorKind {
    DBError(DBError),
    RecordError(RecordError),
    CsvError(CsvError),
    InvalidHeaders,
    #[allow(dead_code)]
    NotEnoughAvailableCredit,
    UnknownTransaction,
}

impl From<DBError> for EngineError {
    fn from(err: DBError) -> EngineError {
        EngineError::new(EngineErrorKind::DBError(err))
    }
}

impl From<RecordError> for EngineError {
    fn from(err: RecordError) -> EngineError {
        EngineError::new(EngineErrorKind::RecordError(err))
    }
}

impl From<CsvError> for EngineError {
    fn from(err: CsvError) -> EngineError {
        EngineError::new(EngineErrorKind::CsvError(err))
    }
}

// impl From<> for EngineError {
//     fn from(err: CsvError) -> EngineError {
//         EngineError::new(EngineErrorKind::CsvError(err))
//     }
// }

impl fmt::Display for EngineError {
    /// We would need more info about the errors occuring here in a real implementation.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            EngineErrorKind::DBError(ref err) => write!(f, "Database error: {:?}", err),
            EngineErrorKind::RecordError(ref _err) => write!(f, "Record parsing error"),
            EngineErrorKind::CsvError(ref _err) => write!(f, "CSV parse error"),
            EngineErrorKind::InvalidHeaders => write!(f, "Invalid headers encountered"),
            EngineErrorKind::NotEnoughAvailableCredit => {
                write!(f, "Not enough available credit to withdraw")
            }
            EngineErrorKind::UnknownTransaction => write!(f, "Unknown Transaction encountered"),
        }
    }
}
