//! Transaction protocol.

use super::record::Record;

/// The kind of transaction we know how to process
/// from a [`Record`].
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TransactionKind {
    /// Add value to a client's account.
    Deposit,
    /// Remove value from a client's account.
    Withdrawal,
    /// Engage a transaction to the dispute process, putting
    /// the amount of a previous transaction in a hold state
    /// until the dispute process is over.
    Dispute,
    /// Resolving a disputed transaction by adding back to
    /// the client's account, from the holding place, the
    /// disputed transaction's amount.
    Resolve,
    /// Resolving a dispute by removing the disputed transaction's
    /// amount from the client's account and resulting freezing the
    /// account.
    Chargeback,
}

impl TransactionKind {
    /// Returns from a byte string a valid transaction if
    /// it's a known one or None if unknown.
    /// We might want to use a `Result` here.
    pub fn new(trans: &[u8]) -> Option<Self> {
        match trans {
            b"deposit" => Some(Self::Deposit),
            b"withdrawal" => Some(Self::Withdrawal),
            b"dispute" => Some(Self::Dispute),
            b"resolve" => Some(Self::Resolve),
            b"chargeback" => Some(Self::Chargeback),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Transaction {
    #[allow(dead_code)]
    kind: TransactionKind,
    client_id: u16,
    amount: f32,
    disputed: bool,
    // We might want to refactor this with an optional value
    // to match the specs about some transaction that doesn't have
    // an amount value. But because it will not save any space we choose
    // to go for the easy way.
    // amount: Option<f32>,
}

impl Transaction {
    #[allow(dead_code)]
    pub fn new(kind: TransactionKind, client_id: u16, amount: f32) -> Self {
        Self {
            kind,
            client_id,
            amount,
            disputed: false,
        }
    }

    pub fn from_record(record: &Record) -> Self {
        Self {
            kind: record.transaction_kind,
            client_id: record.client,
            amount: record.amount,
            disputed: false,
        }
    }

    pub fn amount(&self) -> f32 {
        self.amount
    }

    pub fn client_id(&self) -> u16 {
        self.client_id
    }

    // Set the dispute state of a transaction.
    pub fn set_dispute(&mut self, b: bool) {
        self.disputed = b;
    }

    pub fn is_in_dispute(&self) -> bool {
        self.disputed
    }
}

#[test]
fn test_transaction_parsing() {
    let byte_record: &[u8] = b"withdrawal";
    assert_eq!(
        TransactionKind::new(byte_record),
        Some(TransactionKind::Withdrawal)
    );
}

#[test]
fn test_bad_transaction_parsing() {
    let utt = b"Unknown_transaction_type";
    assert_eq!(TransactionKind::new(utt), None);

    let empty = b"";
    assert_eq!(TransactionKind::new(empty), None);
}
