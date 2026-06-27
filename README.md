This crate provides an implementation of the ObjectId data type as defined in the BSON specification.

## Usage

```bash
cargo add obid
```

```rust
use obid::ObjectId;

// Create a new ObjectId
let id = ObjectId::new();

// Parse an existing ObjectId from its raw string representation
let id: ObjectId = "536f6d652073656372657420".parse().unwrap();

// Generate an ObjectId from raw bytes
let id: ObjectId = ObjectId::try_from([0x53, 0x6f, 0x6d, 0x65, 0x20, 0x73, 0x65, 0x63, 0x72, 0x65, 0x74, 0x20]).expect("invalid ObjectId bytes");

let id: ObjectId = ObjectId::try_from("abcdefghilmn").expect("invalid ObjectId bytes");

// etc...
```

As seen in the examples above, the `try_from` method is used to convert a `&str` or `[u8; 12]` into an `ObjectId` by using the raw bytes from the input as "seed", while parsing means turning a string representation into an `ObjectId`.

## `no_std` Support

To use this crate without the standard library, e.g. for embedded systems, disable the `std` feature.

Please note that when not using the standard library some of the BSON specs
for ObjectId are forcefully ignored. For example, in case a deterministic seed is required,
there is no way to determine it from the PID of the process so `0` is used instead.


## Features
- `std`: Enables use of the standard library.
- `archive`: Enables serialization/deserialization using the `rkyv` crate.
- `serde`: Enables serialization/deserialization using the `serde` crate.
