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

impl std::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_public_key() {
        let key_str = String::from("ki6htfv2252cj2lhq3hxu4qfcfjtpjnukzonevigudzjpmmruxva");
        let fmt_str = String::from("ki6htfv2252cj2lh");
        let bytes = b"\x52\x3c\x79\x96\xba\xd7\x74\x24\xe9\x67\x86\xcf\x7a\x72\x05\x11\x53\x37\xa5\xb4\x56\x5c\xd2\x55\x06\xa0\xf2\x97\xb1\x91\xa5\xea";
        //
        // create key from string
        let key = PublicKey::from_string(key_str.clone()).unwrap();
        //
        // test methods are as expected
        assert_eq!(key_str, key.to_string());
        assert_eq!(bytes.to_vec(), key.to_bytes());
        assert_eq!(fmt_str, key.fmt_short());
        //
        // create key from bytes
        let key_0 = Arc::new(PublicKey::from_bytes(bytes.to_vec()).unwrap());
        //
        // test methods are as expected
        assert_eq!(key_str, key_0.to_string());
        assert_eq!(bytes.to_vec(), key_0.to_bytes());
        assert_eq!(fmt_str, key_0.fmt_short());
        //
        // test that the eq function works
        assert!(key.equal(key_0.clone()));
        assert!(key_0.equal(key.into()));
    }
}
