use std::{
    hash::{Hash, Hasher},
    str::FromStr,
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::IrohError;

/// An endpoint's identifier, a 32-byte ed25519 public key.
///
/// In iroh 1.0 this is an alias for the underlying `PublicKey` cryptographic type
/// and uniquely identifies an [`Endpoint`](crate::Endpoint).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, uniffi::Object)]
#[uniffi::export(Display, Eq, Hash)]
pub struct EndpointId {
    pub(crate) key: [u8; 32],
}

impl From<iroh::EndpointId> for EndpointId {
    fn from(key: iroh::EndpointId) -> Self {
        EndpointId {
            key: *key.as_bytes(),
        }
    }
}

impl From<&EndpointId> for iroh::EndpointId {
    fn from(key: &EndpointId) -> Self {
        iroh::EndpointId::from_bytes(&key.key).expect("EndpointId bytes are always valid")
    }
}

#[uniffi::export]
impl EndpointId {
    /// Get the underlying 32 bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.key.to_vec()
    }

    /// Parse an [`EndpointId`] from its base32 representation.
    #[uniffi::constructor]
    pub fn from_string(s: String) -> Result<Self, IrohError> {
        let key = iroh::EndpointId::from_str(&s)?;
        Ok(key.into())
    }

    /// Construct an [`EndpointId`] from raw bytes.
    #[uniffi::constructor]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, IrohError> {
        if bytes.len() != 32 {
            return Err(IrohError::invalid_input(
                "the EndpointId must be 32 bytes in length",
            ));
        }
        let bytes: [u8; 32] = bytes.try_into().expect("checked above");
        let key = iroh::EndpointId::from_bytes(&bytes)?;
        Ok(key.into())
    }

    /// Short, base32 prefix of the [`EndpointId`].
    pub fn fmt_short(&self) -> String {
        iroh::EndpointId::from(self).fmt_short().to_string()
    }

    /// Verify a signature on `message` against this endpoint's key.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), IrohError> {
        iroh::EndpointId::from(self)
            .verify(message, &signature.0)
            .map_err(|e| anyhow::anyhow!("signature verification failed: {e:?}").into())
    }
}

/// The secret key half of an endpoint identity.
///
/// Mirrors `iroh::SecretKey`. Used internally by [`Endpoint`](crate::Endpoint) to
/// produce its TLS certificate and to sign arbitrary messages.
#[derive(Debug, Clone, uniffi::Object)]
pub struct SecretKey(pub(crate) iroh::SecretKey);

impl From<iroh::SecretKey> for SecretKey {
    fn from(key: iroh::SecretKey) -> Self {
        SecretKey(key)
    }
}

#[uniffi::export]
impl SecretKey {
    /// Generate a new random secret key.
    #[uniffi::constructor]
    pub fn generate() -> Self {
        SecretKey(iroh::SecretKey::generate())
    }

    /// Construct a [`SecretKey`] from raw bytes.
    #[uniffi::constructor]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, IrohError> {
        let bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| IrohError::invalid_input("SecretKey requires exactly 32 bytes"))?;
        Ok(SecretKey(iroh::SecretKey::from_bytes(&bytes)))
    }

    /// Get the underlying 32 bytes of the secret key.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }

    /// The public [`EndpointId`] derived from this secret key.
    pub fn public(&self) -> Arc<EndpointId> {
        Arc::new(self.0.public().into())
    }

    /// Sign a message, producing an ed25519 signature.
    pub fn sign(&self, message: &[u8]) -> Arc<Signature> {
        Arc::new(Signature(self.0.sign(message)))
    }
}

/// An ed25519 signature over a message.
#[derive(Debug, Clone, uniffi::Object)]
#[uniffi::export(Display, Eq, Hash)]
pub struct Signature(pub(crate) iroh_base::Signature);

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for b in self.0.to_bytes() {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

#[uniffi::export]
impl Signature {
    /// Construct a [`Signature`] from raw bytes (64 bytes).
    #[uniffi::constructor]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, IrohError> {
        let bytes: [u8; 64] = bytes
            .try_into()
            .map_err(|_| IrohError::invalid_input("Signature requires exactly 64 bytes"))?;
        Ok(Signature(iroh_base::Signature::from_bytes(&bytes)))
    }

    /// Get the underlying 64 bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }
}

impl Eq for Signature {}
impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bytes() == other.0.to_bytes()
    }
}

impl Hash for Signature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bytes().hash(state)
    }
}

impl std::fmt::Display for EndpointId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        iroh::EndpointId::from(self).fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_id() {
        let key_str =
            String::from("523c7996bad77424e96786cf7a7205115337a5b4565cd25506a0f297b191a5ea");
        let fmt_str = String::from("523c7996ba");
        let bytes = b"\x52\x3c\x79\x96\xba\xd7\x74\x24\xe9\x67\x86\xcf\x7a\x72\x05\x11\x53\x37\xa5\xb4\x56\x5c\xd2\x55\x06\xa0\xf2\x97\xb1\x91\xa5\xea";

        let id = EndpointId::from_string(key_str.clone()).unwrap();
        assert_eq!(key_str, id.to_string());
        assert_eq!(bytes.to_vec(), id.to_bytes());
        assert_eq!(fmt_str, id.fmt_short());

        let id_0 = EndpointId::from_bytes(bytes.to_vec()).unwrap();
        assert_eq!(key_str, id_0.to_string());
        assert_eq!(bytes.to_vec(), id_0.to_bytes());
        assert_eq!(fmt_str, id_0.fmt_short());

        assert_eq!(id, id_0);
        assert_eq!(id_0, id);
    }

    #[test]
    fn test_sign_verify_roundtrip() {
        let secret = SecretKey::generate();
        let id = secret.public();
        let msg = b"hello iroh".to_vec();
        let sig = secret.sign(&msg);
        id.verify(&msg, &sig).unwrap();
    }

    #[test]
    fn test_secret_key_bytes_roundtrip() {
        let secret = SecretKey::generate();
        let bytes = secret.to_bytes();
        let secret2 = SecretKey::from_bytes(bytes.clone()).unwrap();
        assert_eq!(secret.to_bytes(), secret2.to_bytes());
        assert_eq!(secret.public().to_bytes(), secret2.public().to_bytes());
    }

    #[test]
    fn test_error_kind_for_invalid_endpoint_id() {
        let err = EndpointId::from_bytes(vec![0; 31]).unwrap_err();
        assert_eq!(err.kind(), crate::IrohErrorKind::InvalidInput);
        assert!(err.is_kind(crate::IrohErrorKind::InvalidInput));
        assert!(err.message().contains("32 bytes"));
        assert_eq!(err.debug_message(), err.message());

        let err = EndpointId::from_string("not-an-endpoint-id".to_string()).unwrap_err();
        assert_eq!(err.kind(), crate::IrohErrorKind::KeyParsing);
        assert!(err.is_kind(crate::IrohErrorKind::KeyParsing));
        assert!(!err.message().is_empty());
        assert!(!err.debug_message().is_empty());
    }
}
