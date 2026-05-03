use serde::{Deserialize, Serialize};

use crate::ObjectId;

impl Serialize for ObjectId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.as_slice())
    }
}

impl<'de> Deserialize<'de> for ObjectId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        Ok(ObjectId::try_from_slice(&bytes).unwrap())
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let obj_id = ObjectId::default();
        let serialized = serde_json::to_string(&obj_id).unwrap();
        let deserialized: ObjectId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(obj_id, deserialized);
    }
}
