//! Relay configuration surface.
//!
//! Mirrors `iroh::RelayMode`, `iroh::RelayMap`, and `iroh::RelayConfig`. Tickets
//! and `RelayUrl`s use string passthrough — callers pass an HTTPS-style URL and
//! we parse on entry.

use std::{str::FromStr, sync::Arc};

use crate::IrohError;

/// Config for a single relay server.
///
/// `url` must parse as a `RelayUrl` (HTTPS URL). `quic_port` enables QUIC
/// address discovery when set; leaving it `None` disables it. `auth_token`
/// becomes an `Authorization: Bearer ...` header on the upgrade request.
#[derive(Debug, Clone, uniffi::Record)]
pub struct RelayConfig {
    pub url: String,
    #[uniffi(default = None)]
    pub quic_port: Option<u16>,
    #[uniffi(default = None)]
    pub auth_token: Option<String>,
}

impl TryFrom<RelayConfig> for iroh::RelayConfig {
    type Error = IrohError;

    fn try_from(value: RelayConfig) -> Result<Self, Self::Error> {
        let url = iroh::RelayUrl::from_str(&value.url).map_err(anyhow::Error::from)?;
        let quic = value.quic_port.map(iroh_relay::RelayQuicConfig::new);
        let mut config = iroh::RelayConfig::new(url, quic);
        if let Some(token) = value.auth_token {
            config = config.with_auth_token(token);
        }
        Ok(config)
    }
}

impl From<&iroh::RelayConfig> for RelayConfig {
    fn from(value: &iroh::RelayConfig) -> Self {
        Self {
            url: value.url.to_string(),
            quic_port: value.quic.as_ref().map(|q| q.port),
            auth_token: value.auth_token.clone(),
        }
    }
}

/// A collection of relay servers an endpoint should consider.
///
/// Mirrors `iroh::RelayMap`. Construct with [`Self::empty`] or [`Self::from_urls`]
/// and mutate with [`Self::insert`] / [`Self::remove`].
#[derive(Debug, Clone, uniffi::Object)]
#[uniffi::export(Display)]
pub struct RelayMap(pub(crate) iroh::RelayMap);

impl From<iroh::RelayMap> for RelayMap {
    fn from(map: iroh::RelayMap) -> Self {
        Self(map)
    }
}

impl std::fmt::Display for RelayMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let urls: Vec<String> = self
            .0
            .urls::<Vec<_>>()
            .into_iter()
            .map(|u| u.to_string())
            .collect();
        write!(f, "RelayMap([{}])", urls.join(", "))
    }
}

#[uniffi::export]
impl RelayMap {
    /// Create an empty relay map.
    #[uniffi::constructor]
    pub fn empty() -> Self {
        Self(iroh::RelayMap::empty())
    }

    /// Build a relay map from a list of relay URLs (each becomes a default
    /// [`RelayConfig`]).
    #[uniffi::constructor]
    pub fn from_urls(urls: Vec<String>) -> Result<Self, IrohError> {
        let map = iroh::RelayMap::try_from_iter(urls.iter().map(|s| s.as_str()))
            .map_err(anyhow::Error::from)?;
        Ok(Self(map))
    }

    /// Insert a relay (replacing any prior entry for the same URL).
    pub fn insert(&self, config: RelayConfig) -> Result<(), IrohError> {
        let config: iroh::RelayConfig = config.try_into()?;
        let url = config.url.clone();
        self.0.insert(url, Arc::new(config));
        Ok(())
    }

    /// Remove the entry for the given relay URL. Returns true if something was
    /// removed.
    pub fn remove(&self, url: String) -> Result<bool, IrohError> {
        let url = iroh::RelayUrl::from_str(&url).map_err(anyhow::Error::from)?;
        Ok(self.0.remove(&url).is_some())
    }

    /// Check whether the given relay URL is in the map.
    pub fn contains(&self, url: String) -> Result<bool, IrohError> {
        let url = iroh::RelayUrl::from_str(&url).map_err(anyhow::Error::from)?;
        Ok(self.0.contains(&url))
    }

    /// Look up the configuration for the given relay URL.
    pub fn get(&self, url: String) -> Result<Option<RelayConfig>, IrohError> {
        let url = iroh::RelayUrl::from_str(&url).map_err(anyhow::Error::from)?;
        Ok(self.0.get(&url).map(|c| (c.as_ref()).into()))
    }

    /// All relay URLs currently in the map.
    pub fn urls(&self) -> Vec<String> {
        self.0
            .urls::<Vec<_>>()
            .into_iter()
            .map(|u| u.to_string())
            .collect()
    }

    /// Number of relays in the map.
    pub fn len(&self) -> u32 {
        self.0.len() as _
    }

    /// True if the map has no relays.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Configuration for which relay servers an endpoint uses.
///
/// Mirrors `iroh::RelayMode`. Use one of the constructors below.
#[derive(Debug, Clone, uniffi::Object)]
#[uniffi::export(Display)]
pub struct RelayMode(pub(crate) iroh::RelayMode);

impl From<iroh::RelayMode> for RelayMode {
    fn from(mode: iroh::RelayMode) -> Self {
        Self(mode)
    }
}

impl std::fmt::Display for RelayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            iroh::RelayMode::Disabled => write!(f, "disabled"),
            iroh::RelayMode::Default => write!(f, "default"),
            iroh::RelayMode::Staging => write!(f, "staging"),
            iroh::RelayMode::Custom(map) => {
                write!(f, "custom({} relays)", map.len())
            }
        }
    }
}

#[uniffi::export]
impl RelayMode {
    /// No relays — listening and dialing via relay are both disabled.
    #[uniffi::constructor]
    pub fn disabled() -> Self {
        Self(iroh::RelayMode::Disabled)
    }

    /// Use the n0 production relay map.
    #[uniffi::constructor]
    pub fn default_mode() -> Self {
        Self(iroh::RelayMode::Default)
    }

    /// Use the n0 staging relay map.
    #[uniffi::constructor]
    pub fn staging() -> Self {
        Self(iroh::RelayMode::Staging)
    }

    /// Use a custom relay map.
    #[uniffi::constructor]
    pub fn custom(map: &RelayMap) -> Self {
        Self(iroh::RelayMode::Custom(map.0.clone()))
    }

    /// Build a custom relay mode directly from a list of relay URLs.
    #[uniffi::constructor]
    pub fn custom_from_urls(urls: Vec<String>) -> Result<Self, IrohError> {
        let urls: Vec<iroh::RelayUrl> = urls
            .into_iter()
            .map(|s| {
                iroh::RelayUrl::from_str(&s).map_err(|e| IrohError::from(anyhow::Error::from(e)))
            })
            .collect::<Result<Vec<_>, IrohError>>()?;
        Ok(Self(iroh::RelayMode::custom(urls)))
    }

    /// The relay map this mode resolves to.
    pub fn relay_map(&self) -> Arc<RelayMap> {
        Arc::new(self.0.relay_map().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_map_crud() {
        let map = RelayMap::empty();
        assert!(map.is_empty());

        let cfg = RelayConfig {
            url: "https://relay.example.org/".to_string(),
            quic_port: Some(7842),
            auth_token: Some("hunter2".to_string()),
        };
        map.insert(cfg.clone()).unwrap();
        assert_eq!(map.len(), 1);
        assert!(
            map.contains("https://relay.example.org/".to_string())
                .unwrap()
        );

        let got = map
            .get("https://relay.example.org/".to_string())
            .unwrap()
            .unwrap();
        assert_eq!(got.url, cfg.url);
        assert_eq!(got.quic_port, cfg.quic_port);
        assert_eq!(got.auth_token, cfg.auth_token);

        assert!(
            map.remove("https://relay.example.org/".to_string())
                .unwrap()
        );
        assert!(map.is_empty());
    }

    #[test]
    fn test_relay_mode_constructors() {
        let _ = RelayMode::disabled();
        let _ = RelayMode::default_mode();
        let _ = RelayMode::staging();
        let map = RelayMap::from_urls(vec!["https://r1.example.org/".to_string()]).unwrap();
        let _ = RelayMode::custom(&map);
        let _ = RelayMode::custom_from_urls(vec!["https://r2.example.org/".to_string()]).unwrap();
    }
}
