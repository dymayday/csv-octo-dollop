//! Transaction protocol.

// use crate::engine::record;
use super::record::Record;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TransactionKind {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
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

pub type TransactionDB = BTreeMap<u32, Transaction>;

#[derive(Debug)]
pub struct Transaction {
    #[allow(dead_code)]
    kind: TransactionKind,
    client_id: u16,
    amount: f32,
    // We might want to refactor this with an optional value.
    // amount: Option<f32>,
}

impl Transaction {
    pub fn from_record(record: &Record) -> Self {
        Self {
            kind: record.transaction_kind,
            client_id: record.client,
            amount: record.amount,
        }
    }

    pub fn amount(&self) -> f32 {
        self.amount
    }

    pub fn client_id(&self) -> u16 {
        self.client_id
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
