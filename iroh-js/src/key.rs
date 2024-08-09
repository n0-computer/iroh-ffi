use std::str::FromStr;

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// A public key.
///
/// The key itself is just a 32 byte array, but a key has associated crypto
/// information that is cached for performance reasons.
#[derive(Debug, Clone, Eq)]
#[napi]
pub struct PublicKey {
    /// The actual key bytes. Always 32 bytes.
    key: [u8; 32],
}

impl From<iroh::net::key::PublicKey> for PublicKey {
    fn from(key: iroh::net::key::PublicKey) -> Self {
        PublicKey {
            key: *key.as_bytes(),
        }
    }
}

impl From<&PublicKey> for iroh::net::key::PublicKey {
    fn from(key: &PublicKey) -> Self {
        let key: &[u8] = &key.key[..];
        key.try_into().unwrap()
    }
}

#[napi]
impl PublicKey {
    /// Returns true if the PublicKeys are equal
    #[napi]
    pub fn is_equal(&self, other: &PublicKey) -> bool {
        *self == *other
    }

    /// Express the PublicKey as a byte array
    #[napi]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.key.to_vec()
    }

    /// Make a PublicKey from base32 string
    #[napi(factory)]
    pub fn from_string(s: String) -> Result<Self> {
        let key = iroh::net::key::PublicKey::from_str(&s).map_err(anyhow::Error::from)?;
        Ok(key.into())
    }

    /// Make a PublicKey from byte array
    #[napi(factory)]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("the PublicKey must be 32 bytes in length").into());
        }
        let bytes: [u8; 32] = bytes.try_into().expect("checked above");
        let key = iroh::net::key::PublicKey::from_bytes(&bytes).map_err(anyhow::Error::from)?;
        Ok(key.into())
    }

    /// Convert to a base32 string limited to the first 10 bytes for a friendly string
    /// representation of the key.
    #[napi]
    pub fn fmt_short(&self) -> String {
        iroh::net::key::PublicKey::from(self).fmt_short()
    }

    /// Converts the public key into base32 string.
    #[napi]
    pub fn to_string(&self) -> String {
        iroh::net::key::PublicKey::from(self).to_string()
    }
}

impl PartialEq for PublicKey {
    fn eq(&self, other: &PublicKey) -> bool {
        self.key == other.key
    }
}
