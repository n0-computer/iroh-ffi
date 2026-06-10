use std::str::FromStr;

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// An endpoint's identifier, a 32-byte ed25519 public key.
#[derive(Debug, Clone, Eq)]
#[napi]
pub struct EndpointId {
    key: [u8; 32],
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

impl EndpointId {
    pub(crate) fn raw_bytes(&self) -> [u8; 32] {
        self.key
    }

    pub(crate) fn from_raw_bytes(key: [u8; 32]) -> Self {
        EndpointId { key }
    }
}

#[napi]
impl EndpointId {
    /// Returns true if both [`EndpointId`]s are equal.
    #[napi]
    pub fn equals(&self, other: &EndpointId) -> bool {
        *self == *other
    }

    /// Get the underlying 32 bytes.
    #[napi]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.key.to_vec()
    }

    /// Parse an [`EndpointId`] from its base32 representation.
    #[napi(factory)]
    pub fn from_string(s: String) -> Result<Self> {
        let key = iroh::EndpointId::from_str(&s).map_err(anyhow::Error::from)?;
        Ok(key.into())
    }

    /// Construct an [`EndpointId`] from raw bytes (32 bytes).
    #[napi(factory)]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        let bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("EndpointId requires exactly 32 bytes"))?;
        let key = iroh::EndpointId::from_bytes(&bytes).map_err(anyhow::Error::from)?;
        Ok(key.into())
    }

    /// Short base32 prefix.
    #[napi]
    pub fn fmt_short(&self) -> String {
        iroh::EndpointId::from(self).fmt_short().to_string()
    }

    /// Base32 string form.
    #[napi]
    pub fn to_string(&self) -> String {
        iroh::EndpointId::from(self).to_string()
    }

    /// Verify a signature on `message` against this endpoint's key.
    #[napi]
    pub fn verify(&self, message: Vec<u8>, signature: &Signature) -> Result<()> {
        iroh::EndpointId::from(self)
            .verify(&message, &signature.0)
            .map_err(|e| anyhow::anyhow!("signature verification failed: {e:?}").into())
    }
}

impl PartialEq for EndpointId {
    fn eq(&self, other: &EndpointId) -> bool {
        self.key == other.key
    }
}

/// The secret key half of an endpoint identity.
#[derive(Debug, Clone)]
#[napi]
pub struct SecretKey(pub(crate) iroh::SecretKey);

impl From<iroh::SecretKey> for SecretKey {
    fn from(key: iroh::SecretKey) -> Self {
        SecretKey(key)
    }
}

#[napi]
impl SecretKey {
    /// Generate a new random secret key.
    #[napi(factory)]
    pub fn generate() -> Self {
        SecretKey(iroh::SecretKey::generate())
    }

    /// Construct from raw bytes (32 bytes).
    #[napi(factory)]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        let bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("SecretKey requires exactly 32 bytes"))?;
        Ok(SecretKey(iroh::SecretKey::from_bytes(&bytes)))
    }

    /// Raw 32-byte form.
    #[napi]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }

    /// The public [`EndpointId`] derived from this secret key.
    #[napi]
    pub fn public(&self) -> EndpointId {
        self.0.public().into()
    }

    /// Sign a message, producing an ed25519 signature.
    #[napi]
    pub fn sign(&self, message: Vec<u8>) -> Signature {
        Signature(self.0.sign(&message))
    }
}

/// An ed25519 signature over a message.
#[derive(Debug, Clone)]
#[napi]
pub struct Signature(pub(crate) iroh_base::Signature);

#[napi]
impl Signature {
    /// Construct from raw bytes (64 bytes).
    #[napi(factory)]
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        let bytes: [u8; 64] = bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Signature requires exactly 64 bytes"))?;
        Ok(Signature(iroh_base::Signature::from_bytes(&bytes)))
    }

    /// Raw 64-byte form.
    #[napi]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }

    /// Lowercase hex representation.
    #[napi]
    pub fn to_string(&self) -> String {
        self.0
            .to_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}
