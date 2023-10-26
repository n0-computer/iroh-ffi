use std::str::FromStr;
use std::sync::Arc;

use crate::IrohError;

/// A public key.
///
/// The key itself is just a 32 byte array, but a key has associated crypto
/// information that is cached for performance reasons.
#[derive(Debug, Clone, Eq)]
pub struct PublicKey(pub(crate) iroh::net::key::PublicKey);

impl From<iroh::net::key::PublicKey> for PublicKey {
    fn from(key: iroh::net::key::PublicKey) -> Self {
        PublicKey(key)
    }
}

impl PublicKey {
    /// Express the PublicKey as a base32 string
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    /// Returns true if the PublicKeys are equal
    pub fn equal(&self, other: Arc<PublicKey>) -> bool {
        *self == *other
    }

    /// Express the PublicKey as a byte array
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }

    /// Make a PublicKey from base32 string
    pub fn from_string(s: String) -> Result<Self, IrohError> {
        match iroh::net::key::PublicKey::from_str(&s) {
            Ok(key) => Ok(key.into()),
            Err(err) => Err(IrohError::public_key(err)),
        }
    }

    /// Make a PublicKey from byte array
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, IrohError> {
        if bytes.len() != 32 {
            return Err(IrohError::PublicKey {
                description: "the PublicKey must be 32 bytes in length".into(),
            });
        }
        let bytes: [u8; 32] = bytes.try_into().expect("checked above");
        match iroh::net::key::PublicKey::from_bytes(&bytes) {
            Ok(key) => Ok(key.into()),
            Err(err) => Err(IrohError::public_key(err)),
        }
    }

    /// Convert to a base32 string limited to the first 10 bytes for a friendly string
    /// representation of the key.
    pub fn fmt_short(&self) -> String {
        self.0.fmt_short()
    }
}

impl PartialEq for PublicKey {
    fn eq(&self, other: &PublicKey) -> bool {
        self.0 == other.0
    }
}
