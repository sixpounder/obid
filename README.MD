This crate provides an implementation of the ObjectId data type as defined in the BSON specification.

## Usage

```bash
cargo add obid
```

```rust
use obid::ObjectId;

// Create a new ObjectId
let id = ObjectId::new();

// Or parse an existing one
let id: ObjectId = "536f6d652073656372657420".parse().unwrap();
```

## Features
- `archive`: Enables serialization/deserialization using the `rkyv` crate.
- `serde`: Enables serialization/deserialization using the `serde` crate.
