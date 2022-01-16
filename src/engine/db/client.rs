//! Client logic implementation.

use super::error::{DBError, Result};
use std::collections::BTreeMap;

pub type ClientDB = BTreeMap<u16, ClientAccountState>;

#[derive(Debug)]
pub struct ClientAccountState {
    available: f32,
    held: f32,
    total: f32,
    locked: bool,
}

impl ClientAccountState {
    /// Returns a new ClientAccountState from data.
    pub fn new() -> Self {
        // pub fn new(available: f32, held: f32, total: f32, locked: bool) -> Self {
        ClientAccountState {
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }

    /// Returns the available value.
    pub fn available(&self) -> f32 {
        self.available
    }

    pub fn held(&self) -> f32 {
        self.held
    }

    pub fn total(&self) -> f32 {
        self.total
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    /// Add some value to the client's account.
    pub fn add(&mut self, x: f32) -> Result<()> {
        if x >= 0.0 {
            self.available += x;
            self.total += x;
            Ok(())
        } else {
            Err(DBError::NegativeAmountEncountered)
        }
    }

    /// Subtracting value from the client's account if it's possible,
    /// returns an error otherwise.
    /// We choose to denie it if there is not enough credit available
    /// in a client's account.
    pub fn sub(&mut self, x: f32) -> Result<()> {
        if x >= 0.0 {
            self.available -= x;
            self.total -= x;
            Ok(())
        } else {
            Err(DBError::NegativeAmountEncountered)
        }
    }

    /// Locks the client's account during the time of dispute.
    pub fn lock(&mut self) -> Result<()> {
        self.locked = true;
        Ok(())
    }

    #[allow(dead_code)]
    /// Locks the client's account during the time of dispute.
    pub fn unlock(&mut self) -> Result<()> {
        self.locked = false;
        Ok(())
    }

    /// Hold value from the client during dispute.
    pub fn hold(&mut self, x: f32) -> Result<()> {
        self.available -= x;
        self.held += x;
        Ok(())
    }

    // Releases the held funds and add it back to the available
    // amount.
    pub fn unhold(&mut self, x: f32) -> Result<()> {
        // Check if there is a enough held values.
        if self.held >= x {
            self.held -= x;
            self.available += x;
            Ok(())
        } else {
            Err(DBError::NotEnoughHeldValue)
        }
    }
}

#[test]
fn test_sub() {
    let mut cas = ClientAccountState::new();
    cas.add(3.0).unwrap();
    cas.sub(1.0).unwrap();
    assert_eq!(cas.available(), 2.0);
    assert_eq!(cas.total(), 2.0);
}

#[test]
fn test_add() {
    let mut cas = ClientAccountState::new();
    cas.add(10.0).unwrap();
    assert_eq!(cas.available(), 10.0);
    assert_eq!(cas.total(), 10.0);
    assert!(cas.add(-1.0).is_err());
}

#[test]
fn test_hold() {
    let mut cas = ClientAccountState::new();
    cas.add(3.0).unwrap();
    cas.hold(1.0).unwrap();

    assert_eq!(cas.total(), 3.0);
    assert_eq!(cas.held(), 1.0);
}
