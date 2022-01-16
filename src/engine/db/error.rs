//! Database Error implementation.
pub type Result<T> = std::result::Result<T, DBError>;

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum DBError {
    InvalidTransaction,
    OperationNotPermitted,
    NegativeAmountEncountered,
    TransactionAlreadyExists,
    TransactionNotFound,
    NotEnoughAvailableCredit,
    NotEnoughHeldValue,
    ClientNotFound,
    ClientIdMismatch,
}
