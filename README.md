# obid
The `obid` crate is your reliable, fast, and standards-compliant way to work with the BSON `ObjectId` data type in Rust.

This crate makes sure your application handles the unique 12-byte identifier format used in databases like MongoDB correctly.

### Installation

Start by adding the crate to your `Cargo.toml`:

```toml
cargo add obid
```

### Features

You can customize the crate's functionality using feature flags:

| Feature | Description | When to use it |
| :--- | :--- | :--- |
| `std` | Enables full use of the standard library. | For most standard desktop or backend projects. |
| `archive` | Adds serialization/deserialization using the `rkyv` crate. | If you need very high-performance data serialization. |
| `serde` | Adds serialization/deserialization using the `serde` crate. | When integrating with common data formats or APIs. |

### Usage Examples

#### 1. Creating a New ObjectId

Use this to generate a brand new, random `ObjectId`.

```rust
use obid::ObjectId;

// Creates a new ObjectId with a unique value.
let new_id = ObjectId::new();
println!("{}", new_id);
```

#### 2. Parsing from String Representation

Use `ObjectId::parse()` when you have an `ObjectId` that is already represented as its standard 24-character hexadecimal string.

```rust
use obid::ObjectId;

// Parsing a standard hex string (e.g., "536f6d6520736563726574")
let id_from_str: ObjectId = "536f6d6520736563726574".parse().expect("Invalid ObjectId string format");
println!("Parsed ID: {}", id_from_str);
```

#### 3. Creating from Raw Bytes

Use `ObjectId::try_from()` when you have the 12 raw bytes (for instance, if they were read directly from a file or network packet).

```rust
use obid::ObjectId;

// From explicit byte array (e.g., &[u8])
let raw_bytes: [u8; 12] = [0x53, 0x6f, 0x6d, 0x65, 0x20, 0x73, 0x65, 0x63, 0x72, 0x65, 0x74, 0x00];
let id_from_bytes: ObjectId = ObjectId::try_from(&raw_bytes).expect("Invalid ObjectId bytes");
println!("ID from bytes: {}", id_from_bytes);
```

> **Quick Tip:**
>
> *   **`parse()`**: Reads a formatted string and turns it into an `ObjectId`.
> *   **`try_from()`**: Takes raw bytes and tries to build the `ObjectId` from them.

### no_std Support

If you are working in an environment that doesn't have a standard library (like embedded systems), you can disable the `std` feature flag:

```toml
[dependencies]
obid = { version = "x.y.z", default-features = false, features = ["no_std"] }
```

**Beware:** When using `no_std`, some BSON specs that rely on standard library features (like figuring out a deterministic seed from the process PID) might be ignored or just default to `0` to keep things working and deterministic.
