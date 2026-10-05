use uuid::Uuid;
use std::fmt;
use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ObjectId(Uuid);

impl ObjectId {
    #[inline]
    pub fn new() -> Self {
        let bytes: [u8; 16] = rand::random();
        Self(Uuid::from_bytes(bytes))
    }

    #[inline]
    pub fn nil() -> Self {
        Self(Uuid::nil())
    }

    #[inline]
    pub fn null() -> Self {
        Self(Uuid::nil())
    }

    #[inline]
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
    
    #[inline]
    pub fn get_id(&self) -> u64 {
        let bytes = self.0.as_bytes();
        u64::from_ne_bytes(bytes[0..8].try_into().unwrap())
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::str::FromStr for ObjectId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_id_creation() {
        let id = ObjectId::new();
        assert_ne!(id.as_uuid(), &Uuid::nil());
    }

    #[test]
    fn test_object_id_uniqueness() {
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();
        assert_ne!(id1, id2);
    }
}
