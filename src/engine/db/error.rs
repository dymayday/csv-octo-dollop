//! Database Error implementation.
pub type Result<T> = std::result::Result<T, DBError>;

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum DBError {
    #[allow(dead_code)]
    OperationNotPermitted,
    NegativeAmountEncountered,
    TransactionAlreadyExists,
    TransactionNotFound,
    TransactionNotInDispute,
    NotEnoughAvailableCredit,
    NotEnoughHeldValue,
    ClientNotFound,
    ClientIdMismatch,
}
