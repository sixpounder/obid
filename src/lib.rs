#![doc = include_str!("../README.md")]

mod byte_gen;

#[cfg(feature = "serde")]
mod serde;

use std::{
    array::TryFromSliceError,
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, Utc};
use rand::Rng;
use thiserror::Error;

use crate::byte_gen::next_3byte_be;

pub(crate) const OBJECT_ID_LENGTH: usize = 12;

/// An implementation of the ObjectId data type as defined in the BSON specification.
///
/// An ObjectId is a 12-byte value consisting of a 4-byte timestamp, a 5-byte random value, and a 3-byte counter.
#[repr(C)]
#[cfg_attr(
    feature = "archive",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, rkyv::Portable)
)]
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct ObjectId {
    ts: [u8; 4],
    rnd: [u8; 5],
    prg: [u8; 3],
}

impl ObjectId {
    /// Creates a new ObjectId
    pub fn new() -> Self {
        Self::with_timestamp_bytes(unix_seconds_be4().unwrap())
    }

    /// Parses a hexadecimal string into an ObjectId.
    ///
    /// Returns an error if the string is not a valid hexadecimal representation of an ObjectId.
    pub fn parse<S: AsRef<str>>(s: S) -> Result<Self, ObjectIdError> {
        if let Ok(bytes) = hex_to_bytes(s.as_ref()) {
            Self::try_from_slice(&bytes).map_err(|_| ObjectIdError::Parse(s.as_ref().to_string()))
        } else {
            Err(ObjectIdError::Parse(s.as_ref().to_string()))
        }
    }

    /// Creates an ObjectId with the given timestamp in seconds.
    #[allow(dead_code)]
    fn with_timestamp_seconds(seconds: u32) -> Self {
        Self::with_timestamp_bytes(u32::to_be_bytes(seconds))
    }

    /// Creates an ObjectId with the given timestamp in big-endian bytes.
    fn with_timestamp_bytes(ts: [u8; 4]) -> Self {
        let rnd = rand_bytes(5).try_into().unwrap();

        Self {
            ts,
            rnd,
            prg: next_3byte_be(),
        }
    }

    /// Returns the timestamp component of the ObjectId as a `u32` in big-endian order.
    pub fn seconds(&self) -> u32 {
        u32::from_be_bytes(self.ts)
    }

    /// Returns the timestamp component of the ObjectId as a `DateTime<Utc>`.
    pub fn timestamp(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_secs(self.seconds() as i64).expect("invalid timestamp")
    }

    /// Parses an ObjectId from a slice of bytes.
    fn try_from_slice(slice: &[u8]) -> Result<ObjectId, ObjectIdError> {
        if slice.len() < OBJECT_ID_LENGTH {
            return Err(ObjectIdError::InvalidSourceLength(slice.len()));
        }

        let mut ts = [0u8; 4];
        ts.copy_from_slice(&slice[..4]);

        let mut rnd = [0u8; 5];
        rnd.copy_from_slice(&slice[4..9]);

        let mut prg = [0u8; 3];
        prg.copy_from_slice(&slice[9..12]);

        Ok(Self { ts, rnd, prg })
    }

    /// Return a read-only slice of the 12 bytes composing the ObjectId.
    /// Requires that the struct size equals the sum of its fields (12).
    pub fn as_slice(&self) -> &[u8; OBJECT_ID_LENGTH] {
        // Verify at compile / runtime that layout matches expectation.
        debug_assert_eq!(size_of::<Self>(), 12);
        unsafe { &*(self as *const Self as *const [u8; OBJECT_ID_LENGTH]) }
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for ObjectId {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl PartialOrd for ObjectId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.ts.cmp(&other.ts) {
            std::cmp::Ordering::Equal => Some(self.prg.cmp(&other.prg)),
            other => Some(other),
        }
    }
}

impl AsRef<[u8]> for ObjectId {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl AsRef<str> for ObjectId {
    fn as_ref(&self) -> &str {
        str::from_utf8(self.as_slice())
            .expect("Failed to convert to string slice, non utf-8 encoding is not supported")
    }
}

impl Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.as_slice()
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>()
        )
    }
}

impl TryFrom<&[u8]> for ObjectId {
    type Error = ObjectIdError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from_slice(value)
    }
}

impl TryFrom<&[u8; OBJECT_ID_LENGTH]> for ObjectId {
    type Error = ObjectIdError;

    fn try_from(value: &[u8; OBJECT_ID_LENGTH]) -> Result<Self, Self::Error> {
        Self::try_from_slice(value)
    }
}

impl TryFrom<&str> for ObjectId {
    type Error = ObjectIdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes = value.as_bytes();
        if bytes.len() < OBJECT_ID_LENGTH {
            return Err(ObjectIdError::InvalidSourceLength(bytes.len()));
        }

        Self::try_from_slice(bytes)
    }
}

impl TryFrom<String> for ObjectId {
    type Error = ObjectIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl FromStr for ObjectId {
    type Err = ObjectIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ObjectId::parse(s)
    }
}

/// Return a vector (slice-able) of `len` cryptographically-random bytes.
pub fn rand_bytes(len: usize) -> Vec<u8> {
    let mut buf = vec![0u8; len];
    rand::rng().fill_bytes(&mut buf);
    buf
}

/// Return the current Unix seconds as a 4-byte array with the seconds
/// in the highest-order bytes (big-endian).
fn unix_seconds_be4() -> Result<[u8; 4], ObjectIdError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ObjectIdError::InvalidSeed)?;
    let secs = now.as_secs();
    if secs > u32::MAX as u64 {
        return Err(ObjectIdError::SeedOverflow);
    }
    Ok((secs as u32).to_be_bytes())
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    let s = hex
        .strip_prefix("0x")
        .or_else(|| hex.strip_prefix("0X"))
        .unwrap_or(hex);
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

/// Represents an error that can occur when creating an ObjectId.
#[derive(Debug, Clone, Error)]
pub enum ObjectIdError {
    #[error("invalid object id length: {0}")]
    InvalidSourceLength(usize),

    #[error("could not convert from slice: {0}")]
    FromSlice(#[from] TryFromSliceError),

    #[error("seed overflow")]
    SeedOverflow,

    #[error("invalid seed")]
    InvalidSeed,

    #[error("parse error: {0}")]
    Parse(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_structural_sanity() {
        let id = ObjectId::new();
        let reverse = ObjectId::try_from_slice(id.as_slice()).unwrap();
        assert_eq!(id, reverse);

        let id = ObjectId::new();
        let reverse = id.to_string().parse().unwrap();
        let from_literal_bytes = ObjectId::try_from(id.to_string()).unwrap();
        assert_eq!(id, reverse);
        assert_ne!(id, from_literal_bytes);
    }

    /// Tests that an ObjectId can be created from cypher
    #[test]
    fn test_object_id_from_string_slice() {
        let cypher = "Some secret phrase here";
        let id = ObjectId::try_from(cypher);
        assert!(id.is_ok());
        let created_id = id.unwrap();
        dbg!(&created_id.to_string());
        assert_eq!(created_id.clone().to_string(), "536f6d652073656372657420");
    }

    /// Tests that an ObjectId cannot be created from a too short cypher
    #[test]
    fn test_object_id_from_string_slice_short() {
        let cypher = "short";
        let id = ObjectId::try_from(cypher.to_string());
        assert!(matches!(
            id.unwrap_err(),
            ObjectIdError::InvalidSourceLength(_)
        ))
    }

    #[test]
    fn test_object_id_seconds() {
        let id = ObjectId::default();
        assert_ne!(id.seconds(), 0);
    }

    /// Refer to https://specifications.readthedocs.io/en/latest/bson-objectid/objectid/#test-plan
    #[test]
    fn test_object_id_timestamp() {
        assert_eq!(
            ObjectId::with_timestamp_seconds(0x00000000)
                .timestamp()
                .to_string(),
            "1970-01-01 00:00:00 UTC"
        );
        assert_eq!(
            ObjectId::with_timestamp_seconds(0x7FFFFFFF)
                .timestamp()
                .to_string(),
            "2038-01-19 03:14:07 UTC"
        );
        assert_eq!(
            ObjectId::with_timestamp_seconds(0x80000000)
                .timestamp()
                .to_string(),
            "2038-01-19 03:14:08 UTC"
        );
        assert_eq!(
            ObjectId::with_timestamp_seconds(0xFFFFFFFF)
                .timestamp()
                .to_string(),
            "2106-02-07 06:28:15 UTC"
        );
    }

    /// Tests that subsequent ObjectId creation is ordered by timestamp and, if that's equal, by its progressive counter too
    #[test]
    fn subsequent_creation_ordering() {
        let first = ObjectId::default();
        let second = ObjectId::default();

        dbg!(&first, &second);
        assert!(first < second);
        assert!(first != second);
        assert!(second >= first);
    }

    #[test]
    fn test_object_id_cmp() {
        assert!(
            ObjectId::with_timestamp_seconds(0x00000000)
                < ObjectId::with_timestamp_seconds(0x7FFFFFFF)
        );
        assert!(
            ObjectId::with_timestamp_seconds(0x7FFFFFFF)
                < ObjectId::with_timestamp_seconds(0x80000000)
        );
        assert!(
            ObjectId::with_timestamp_seconds(0x80000000)
                < ObjectId::with_timestamp_seconds(0xFFFFFFFF)
        );
    }
}
