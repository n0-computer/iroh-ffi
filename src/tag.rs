use std::sync::Arc;

/// A tag
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag(pub(crate) iroh::bytes::Tag);

impl Tag {
    /// Create a tag from a String
    pub fn from_string(t: String) -> Self {
        let tag: iroh::bytes::Tag = t.into();
        tag.into()
    }

    /// Create a tag from a byte array
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let tag: iroh::bytes::Tag = bytes::Bytes::from(bytes).into();
        tag.into()
    }

    /// Serialize a tag as a byte array
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0 .0.to_vec()
    }

    /// Returns true if the Tags have the same value
    pub fn equal(&self, other: Arc<Tag>) -> bool {
        *self == *other
    }
}

impl From<iroh::bytes::Tag> for Tag {
    fn from(t: iroh::bytes::Tag) -> Self {
        Tag(t)
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Tag> for iroh::bytes::Tag {
    fn from(value: Tag) -> Self {
        value.0
    }
}
