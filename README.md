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

## `no_std` Support

To use this crate without the standard library, e.g. for embedded systems, disable the `std` feature.

Please note that when not using the standard library some of the BSON specs
for ObjectId are forcefully ignore. For example, in case a deterministic seed is required,
there is no way to determine it from the PID of the process so `0` is used instead.


## Features
- `std`: Enables use of the standard library.
- `archive`: Enables serialization/deserialization using the `rkyv` crate.
- `serde`: Enables serialization/deserialization using the `serde` crate.
