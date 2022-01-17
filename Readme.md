# Coding test notes


## Thing to discuss

- This current implementation is based around speed and memory efficiency, as so we whoose to work as close to the metal as possible and [Serde](https://crates.io/crates/serde) has been put aside as it would have occured a performance penalty that we judge unnecessary (see [https://docs.rs/csv/latest/csv/tutorial/index.html#serde-and-zero-allocation](Parsing csv with Serde and zero allocation) for more details about performance).
- Should a frozen client's account state occuring after a `Chargeback` affect all other transaction type, or a `Deposit` can be allowed ?