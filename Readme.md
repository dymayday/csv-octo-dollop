# Coding test notes

A sample data can be found in the example directory.

## Thing to discuss

- This current implementation is based around speed and memory efficiency, as so we whoose to work as close to the metal as possible and [Serde](https://crates.io/crates/serde) has been put aside as it would have occured a performance penalty that we judge unnecessary (see [https://docs.rs/csv/latest/csv/tutorial/index.html#serde-and-zero-allocation](Parsing csv with Serde and zero allocation) for more details about performance).
- Should a frozen client's account state occuring after a `Chargeback` affect all other transaction type, or a `Deposit` is allowed ?
- We chose not to use any async because the order of the transations matters. In a server/clients case this would need refactoring.
- We chose to add a check to discard any `Withdrawal` if there is not enough available amount in a client's account. It might need some thought as an ATM in some cases does allow it.
- We use the type system to ensure the correctness when parsing.
- For performance reason we used an unsafe parsing method of numerical values, but it's fine because we are protected by the type system and infinity check are in place.