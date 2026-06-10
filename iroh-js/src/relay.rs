use std::{str::FromStr, sync::Arc};

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Config for a single relay server.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct RelayConfig {
    pub url: String,
    pub quic_port: Option<u16>,
    pub auth_token: Option<String>,
}

impl TryFrom<RelayConfig> for iroh::RelayConfig {
    type Error = anyhow::Error;

    fn try_from(value: RelayConfig) -> anyhow::Result<Self> {
        let url = iroh::RelayUrl::from_str(&value.url)?;
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
#[derive(Debug, Clone)]
#[napi]
pub struct RelayMap(pub(crate) iroh::RelayMap);

impl From<iroh::RelayMap> for RelayMap {
    fn from(map: iroh::RelayMap) -> Self {
        Self(map)
    }
}

#[napi]
impl RelayMap {
    /// Create an empty relay map.
    #[napi(factory)]
    pub fn empty() -> Self {
        Self(iroh::RelayMap::empty())
    }

    /// Build a relay map from a list of relay URLs.
    #[napi(factory)]
    pub fn from_urls(urls: Vec<String>) -> Result<Self> {
        let map = iroh::RelayMap::try_from_iter(urls.iter().map(|s| s.as_str()))
            .map_err(anyhow::Error::from)?;
        Ok(Self(map))
    }

    /// Insert a relay (replacing any prior entry for the same URL).
    #[napi]
    pub fn insert(&self, config: RelayConfig) -> Result<()> {
        let config: iroh::RelayConfig = config.try_into()?;
        let url = config.url.clone();
        self.0.insert(url, Arc::new(config));
        Ok(())
    }

    /// Remove the entry for the given relay URL. Returns true if removed.
    #[napi]
    pub fn remove(&self, url: String) -> Result<bool> {
        let url = iroh::RelayUrl::from_str(&url).map_err(anyhow::Error::from)?;
        Ok(self.0.remove(&url).is_some())
    }

    /// Check whether the given relay URL is in the map.
    #[napi]
    pub fn contains(&self, url: String) -> Result<bool> {
        let url = iroh::RelayUrl::from_str(&url).map_err(anyhow::Error::from)?;
        Ok(self.0.contains(&url))
    }

    /// Look up the configuration for the given relay URL.
    #[napi]
    pub fn get(&self, url: String) -> Result<Option<RelayConfig>> {
        let url = iroh::RelayUrl::from_str(&url).map_err(anyhow::Error::from)?;
        Ok(self.0.get(&url).map(|c| (c.as_ref()).into()))
    }

    /// All relay URLs currently in the map.
    #[napi]
    pub fn urls(&self) -> Vec<String> {
        self.0
            .urls::<Vec<_>>()
            .into_iter()
            .map(|u| u.to_string())
            .collect()
    }

    /// Number of relays in the map.
    #[napi]
    pub fn len(&self) -> u32 {
        self.0.len() as _
    }

    /// True if the map has no relays.
    #[napi]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Clean string representation.
    #[napi]
    pub fn to_string(&self) -> String {
        let urls: Vec<String> = self
            .0
            .urls::<Vec<_>>()
            .into_iter()
            .map(|u| u.to_string())
            .collect();
        format!("RelayMap([{}])", urls.join(", "))
    }
}

/// Configuration for which relay servers an endpoint uses.
#[derive(Debug, Clone)]
#[napi]
pub struct RelayMode(pub(crate) iroh::RelayMode);

impl From<iroh::RelayMode> for RelayMode {
    fn from(mode: iroh::RelayMode) -> Self {
        Self(mode)
    }
}

#[napi]
impl RelayMode {
    /// No relays.
    #[napi(factory)]
    pub fn disabled() -> Self {
        Self(iroh::RelayMode::Disabled)
    }

    /// Use the n0 production relay map.
    #[napi(factory)]
    pub fn default_mode() -> Self {
        Self(iroh::RelayMode::Default)
    }

    /// Use the n0 staging relay map.
    #[napi(factory)]
    pub fn staging() -> Self {
        Self(iroh::RelayMode::Staging)
    }

    /// Use a custom relay map.
    #[napi(factory)]
    pub fn custom(map: &RelayMap) -> Self {
        Self(iroh::RelayMode::Custom(map.0.clone()))
    }

    /// Build a custom relay mode from a list of relay URLs.
    #[napi(factory)]
    pub fn custom_from_urls(urls: Vec<String>) -> Result<Self> {
        let urls: Vec<iroh::RelayUrl> = urls
            .into_iter()
            .map(|s| iroh::RelayUrl::from_str(&s).map_err(anyhow::Error::from))
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(Self(iroh::RelayMode::custom(urls)))
    }

    /// The relay map this mode resolves to.
    #[napi]
    pub fn relay_map(&self) -> RelayMap {
        self.0.relay_map().into()
    }

    /// Clean string representation.
    #[napi]
    pub fn to_string(&self) -> String {
        match &self.0 {
            iroh::RelayMode::Disabled => "disabled".to_string(),
            iroh::RelayMode::Default => "default".to_string(),
            iroh::RelayMode::Staging => "staging".to_string(),
            iroh::RelayMode::Custom(map) => format!("custom({} relays)", map.len()),
        }
    }
}
