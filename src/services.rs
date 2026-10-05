//! Binding for `iroh-services` — push metrics to services.iroh.computer.
//!
//! Mirrors `iroh_services::Client`. Construct via [`ServicesClient::create`] with
//! a built [`Endpoint`] plus credentials supplied through [`ServicesOptions`].
//!
//! [`preset_iroh_services`] mirrors `iroh_services::preset()`: an endpoint
//! preset that points at your project's dedicated relays and authenticates to
//! them with a token minted from your API key.

use std::{sync::Arc, time::Duration};

use iroh_services::{Client, ClientBuilder};

use crate::{Endpoint, EndpointBuilder, IrohError, Preset};

/// Build options for [`ServicesClient`].
///
/// Supply *exactly one* of `api_secret`, `api_secret_from_env`, or
/// `ssh_key_pem` for the credential. `api_secret_from_env` (when true) reads
/// the `IROH_SERVICES_API_SECRET` environment variable. If a name is provided
/// it is registered with the service; the name must be 2–128 UTF-8 bytes.
#[derive(derive_more::Debug, Default, uniffi::Record)]
pub struct ServicesOptions {
    /// Encoded API secret string (`services1...`). Sets both the remote endpoint
    /// to dial and the per-client capability.
    #[uniffi(default = None)]
    pub api_secret: Option<String>,
    /// If true, read the API secret from `IROH_SERVICES_API_SECRET`.
    #[uniffi(default = None)]
    pub api_secret_from_env: Option<bool>,
    /// Unencrypted PEM-encoded OpenSSH ed25519 private key. Grants full
    /// capabilities; used by node operators / project owners.
    #[uniffi(default = None)]
    pub ssh_key_pem: Option<String>,
    /// Optional endpoint name to register cloud-side.
    #[uniffi(default = None)]
    pub name: Option<String>,
    /// How often (in milliseconds) to push metrics to the service. `0` disables
    /// automatic interval pushes; if omitted the upstream default applies.
    #[uniffi(default = None)]
    pub metrics_interval_ms: Option<u64>,
}

/// Options for [`preset_iroh_services`].
///
/// Supply *exactly one* of `api_secret` or `api_secret_from_env`.
#[derive(derive_more::Debug, Default, uniffi::Record)]
pub struct ServicesPresetOptions {
    /// Your project's relay URLs. Defaults to the n0 public relays when
    /// omitted, matching `iroh_services::preset()`. Passing an empty list is an
    /// error rather than a silent fallback — that is nearly always a filtered
    /// list that came back empty.
    #[uniffi(default = None)]
    pub relays: Option<Vec<String>>,
    /// Encoded API secret string (`services1...`). The relay access token is
    /// minted from this.
    #[uniffi(default = None)]
    pub api_secret: Option<String>,
    /// If true, read the API secret from `IROH_SERVICES_API_SECRET`.
    #[uniffi(default = None)]
    pub api_secret_from_env: Option<bool>,
    /// The endpoint's own identity key (32 bytes) — not your API secret. The
    /// access token is scoped to it, so pass the same key you persist for your
    /// endpoint's identity. A fresh key is generated when omitted.
    ///
    /// Set the key *here*, not via `EndpointOptions::secret_key` or
    /// `EndpointBuilder::secret_key`: both are layered on top of the preset and
    /// would replace the one the token is scoped to. Doing that is an error,
    /// not a silent auth failure — this preset pins the key.
    #[uniffi(default = None)]
    pub endpoint_secret_key: Option<Vec<u8>>,
}

/// Wraps `iroh_services::IrohServicesPreset` as a foreign-visible [`Preset`].
struct ServicesPreset(iroh_services::IrohServicesPreset);

impl Preset for ServicesPreset {
    fn apply(&self, builder: Arc<EndpointBuilder>) {
        builder.apply_iroh_preset(self.0.clone());
        // The access token is scoped to the key the preset just set, so a later
        // `secret_key` call must fail rather than silently break relay auth.
        builder.pin_secret_key();
    }
}

/// Build an endpoint preset for your project's dedicated relays.
///
/// Mirrors `iroh_services::preset()`: mints a short-lived access token scoped to
/// the endpoint's key and to relay use only, then configures the endpoint to use
/// your relays with that token. Pass the result as `EndpointOptions::preset`.
///
/// The token is minted here, at preset-build time, so build the preset shortly
/// before binding the endpoint.
#[uniffi::export]
pub fn preset_iroh_services(options: ServicesPresetOptions) -> Result<Arc<dyn Preset>, IrohError> {
    let mut builder = iroh_services::preset();

    builder = match (
        options.api_secret,
        options.api_secret_from_env.unwrap_or(false),
    ) {
        (Some(_), true) => {
            return Err(anyhow::anyhow!(
                "ServicesPresetOptions: supply only one of api_secret / api_secret_from_env"
            )
            .into());
        }
        (None, false) => {
            return Err(anyhow::anyhow!(
                "ServicesPresetOptions requires one of api_secret or api_secret_from_env=true"
            )
            .into());
        }
        (Some(secret), false) => builder
            .api_secret_from_str(&secret)
            .map_err(|e| anyhow::anyhow!("invalid api secret: {e:?}"))?,
        (None, true) => builder
            .api_secret_from_env()
            .map_err(|e| anyhow::anyhow!("api secret env var: {e:?}"))?,
    };

    // Omitted relays keep the builder's n0 default; an empty list does not.
    if let Some(relays) = options.relays {
        if relays.is_empty() {
            return Err(anyhow::anyhow!(
                "ServicesPresetOptions: relays is empty; omit it to use the n0 relays"
            )
            .into());
        }
        builder = builder
            .relays(relays)
            .map_err(|e| anyhow::anyhow!("invalid relay url: {e:?}"))?;
    }

    if let Some(bytes) = options.endpoint_secret_key {
        let key: [u8; 32] = AsRef::<[u8]>::as_ref(&bytes).try_into().map_err(|e| {
            IrohError::invalid_input(format!("invalid endpoint secret key length: {e:?}"))
        })?;
        builder = builder.secret_key(iroh::SecretKey::from_bytes(&key));
    }

    let preset = builder
        .build()
        .map_err(|e| anyhow::anyhow!("services preset build failed: {e:?}"))?;
    Ok(Arc::new(ServicesPreset(preset)))
}

/// Flattened summary of an `iroh_services::net_diagnostics::DiagnosticsReport`.
///
/// Net-report and portmap details are dropped from the FFI surface (they have
/// deep, non-uniffi-friendly shapes); use the iroh-services dashboard to read
/// the full report after `submit_network_diagnostics(send=true)`.
#[derive(Debug, Clone, uniffi::Record)]
pub struct DiagnosticsSummary {
    /// Endpoint id of the local endpoint.
    pub endpoint_id: String,
    /// Direct addresses (ip:port) that the endpoint reports.
    pub direct_addrs: Vec<String>,
    /// iroh crate version this report was produced with.
    pub iroh_version: String,
    /// iroh-services crate version this report was produced with.
    pub iroh_services_version: String,
    /// True if the local net-report probe returned a result.
    pub has_net_report: bool,
    /// UPnP availability, if a portmap probe was run.
    pub upnp: Option<bool>,
    /// PCP availability, if a portmap probe was run.
    pub pcp: Option<bool>,
    /// NAT-PMP availability, if a portmap probe was run.
    pub nat_pmp: Option<bool>,
}

impl From<iroh_services::net_diagnostics::DiagnosticsReport> for DiagnosticsSummary {
    fn from(r: iroh_services::net_diagnostics::DiagnosticsReport) -> Self {
        let (upnp, pcp, nat_pmp) = match r.portmap_probe {
            Some(p) => (Some(p.upnp), Some(p.pcp), Some(p.nat_pmp)),
            None => (None, None, None),
        };
        Self {
            endpoint_id: r.endpoint_id.to_string(),
            direct_addrs: r.direct_addrs.into_iter().map(|s| s.to_string()).collect(),
            iroh_version: r.iroh_version,
            iroh_services_version: r.iroh_services_version,
            has_net_report: r.net_report.is_some(),
            upnp,
            pcp,
            nat_pmp,
        }
    }
}

/// Client for services.iroh.computer.
///
/// Construct with [`Self::create`]; metrics are pushed automatically while the
/// client is alive. Drop the client (or let it go out of scope) to stop.
#[derive(Clone, uniffi::Object)]
pub struct ServicesClient {
    inner: Client,
}

#[uniffi::export]
impl ServicesClient {
    /// Build a new client bound to the given endpoint.
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn create(endpoint: &Endpoint, options: ServicesOptions) -> Result<Self, IrohError> {
        let mut builder: ClientBuilder = Client::builder(endpoint.raw());

        let creds_set = [
            options.api_secret.is_some(),
            options.api_secret_from_env.unwrap_or(false),
            options.ssh_key_pem.is_some(),
        ]
        .into_iter()
        .filter(|x| *x)
        .count();
        if creds_set == 0 {
            return Err(anyhow::anyhow!(
                "ServicesOptions requires one of api_secret, api_secret_from_env=true, or ssh_key_pem"
            )
            .into());
        }
        if creds_set > 1 {
            return Err(anyhow::anyhow!(
                "ServicesOptions: supply only one of api_secret / api_secret_from_env / ssh_key_pem"
            )
            .into());
        }

        if let Some(secret) = options.api_secret {
            builder = builder
                .api_secret_from_str(&secret)
                .map_err(|e| anyhow::anyhow!("invalid api secret: {e:?}"))?;
        } else if options.api_secret_from_env.unwrap_or(false) {
            builder = builder
                .api_secret_from_env()
                .map_err(|e| anyhow::anyhow!("api secret env var: {e:?}"))?;
        } else if let Some(pem) = options.ssh_key_pem {
            builder = builder
                .ssh_key(&pem)
                .map_err(|e| anyhow::anyhow!("invalid ssh key: {e:?}"))?;
        }

        if let Some(name) = options.name {
            builder = builder
                .name(name)
                .map_err(|e| anyhow::anyhow!("invalid name: {e:?}"))?;
        }
        if let Some(ms) = options.metrics_interval_ms {
            if ms == 0 {
                builder = builder.disable_metrics_interval();
            } else {
                builder = builder.metrics_interval(Duration::from_millis(ms));
            }
        }

        let inner = builder
            .build()
            .await
            .map_err(|e| anyhow::anyhow!("services build failed: {e:?}"))?;
        Ok(ServicesClient { inner })
    }

    /// Read the current endpoint name from the local client.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn name(&self) -> Result<Option<String>, IrohError> {
        self.inner
            .name()
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}").into())
    }

    /// Set the endpoint name cloud-side. Must be 2–128 UTF-8 bytes.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn set_name(&self, name: String) -> Result<(), IrohError> {
        self.inner
            .set_name(name)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}").into())
    }

    /// Ping the remote service to confirm connectivity.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn ping(&self) -> Result<(), IrohError> {
        self.inner
            .ping()
            .await
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("{e:?}").into())
    }

    /// Push the current metrics snapshot now. (Metrics are also pushed on the
    /// interval configured at build time; this lets you force a flush.)
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn push_metrics(&self) -> Result<(), IrohError> {
        self.inner
            .push_metrics()
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}").into())
    }

    /// Run a local network-diagnostics report. When `send` is true the report
    /// is also submitted to iroh-services for storage.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn submit_network_diagnostics(
        &self,
        send: bool,
    ) -> Result<DiagnosticsSummary, IrohError> {
        let report = self
            .inner
            .net_diagnostics(send)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(report.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Endpoint, EndpointOptions};

    /// A well-formed (but fake) `services1...` API secret. The remote it points
    /// at does not exist, so no connection will ever succeed — but the client
    /// connects lazily, so construction must still succeed. This validates the
    /// whole options -> builder -> client plumbing without network.
    const FAKE_API_SECRET: &str = "servicesaaqaobyha4dqobyha4dqobyha4dqobyha4dqobyha4dqobyha4dqob75c4sdqwvay5nwj63yzvqc7iozsh66x53lcpcy5vyc5ledl2pwdaaa";

    async fn minimal_endpoint() -> Endpoint {
        Endpoint::bind(EndpointOptions {
            preset: Some(crate::preset_minimal()),
            ..Default::default()
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_services_client_boots_with_fake_secret() {
        let ep = minimal_endpoint().await;
        let client = ServicesClient::create(
            &ep,
            ServicesOptions {
                api_secret: Some(FAKE_API_SECRET.to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("client should construct (lazy connection)");
        // Drop the client; never call ping() — that needs a live service.
        drop(client);
        ep.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_services_client_rejects_no_credentials() {
        let ep = minimal_endpoint().await;
        let res = ServicesClient::create(&ep, ServicesOptions::default()).await;
        assert!(res.is_err(), "must reject when no credential is supplied");
        ep.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_services_client_rejects_two_credentials() {
        let ep = minimal_endpoint().await;
        let res = ServicesClient::create(
            &ep,
            ServicesOptions {
                api_secret: Some(FAKE_API_SECRET.to_string()),
                api_secret_from_env: Some(true),
                ..Default::default()
            },
        )
        .await;
        assert!(res.is_err(), "must reject when >1 credentials supplied");
        ep.close().await.unwrap();
    }

    /// Options with a valid credential + relay, for tests that vary one field.
    fn preset_options() -> ServicesPresetOptions {
        ServicesPresetOptions {
            relays: Some(vec!["https://relay.example.org/".to_string()]),
            api_secret: Some(FAKE_API_SECRET.to_string()),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_services_preset_binds_with_custom_relays() {
        let preset = preset_iroh_services(preset_options()).expect("preset should build");

        // No connection is attempted at bind time, so the unreachable relay is
        // fine — this checks the mint -> relay map -> builder plumbing.
        let ep = Endpoint::bind(EndpointOptions {
            preset: Some(preset),
            ..Default::default()
        })
        .await
        .unwrap();
        assert!(!ep.bound_sockets().is_empty());
        ep.close().await.unwrap();
    }

    #[test]
    fn test_services_preset_rejects_bad_credentials() {
        assert!(
            preset_iroh_services(ServicesPresetOptions {
                api_secret: None,
                ..preset_options()
            })
            .is_err(),
            "must reject when no credential is supplied"
        );
        assert!(
            preset_iroh_services(ServicesPresetOptions {
                api_secret_from_env: Some(true),
                ..preset_options()
            })
            .is_err(),
            "must reject when >1 credentials supplied"
        );
        assert!(
            preset_iroh_services(ServicesPresetOptions {
                api_secret: Some("not-a-valid-ticket".to_string()),
                ..preset_options()
            })
            .is_err(),
            "must reject a malformed api key"
        );
    }

    #[tokio::test]
    async fn test_services_preset_pins_the_endpoint_key() {
        let preset = preset_iroh_services(preset_options()).unwrap();

        // The token is scoped to the preset's key, so an EndpointOptions key —
        // layered on top of the preset — must fail loudly, not break relay auth
        // at connect time.
        let res = Endpoint::bind(EndpointOptions {
            preset: Some(preset),
            secret_key: Some(vec![7u8; 32]),
            ..Default::default()
        })
        .await;
        assert!(res.is_err(), "must reject a key set outside the preset");
    }

    #[tokio::test]
    async fn test_services_preset_pins_even_identical_key() {
        // The pin is unconditional: even bit-identical bytes are rejected. The
        // policy is "set it in exactly one place," not "set it to the same
        // value" — the latter invites a future reader to relax the guard.
        let key_bytes = vec![7u8; 32];
        let preset = preset_iroh_services(ServicesPresetOptions {
            endpoint_secret_key: Some(key_bytes.clone()),
            ..preset_options()
        })
        .unwrap();
        let res = Endpoint::bind(EndpointOptions {
            preset: Some(preset),
            secret_key: Some(key_bytes),
            ..Default::default()
        })
        .await;
        assert!(res.is_err(), "identical key must still be rejected");
    }

    #[test]
    fn test_services_preset_rejects_short_endpoint_key() {
        assert!(
            preset_iroh_services(ServicesPresetOptions {
                endpoint_secret_key: Some(vec![0u8; 16]),
                ..preset_options()
            })
            .is_err(),
            "must reject an endpoint key that is not 32 bytes"
        );
    }

    #[test]
    fn test_services_preset_relays() {
        assert!(
            preset_iroh_services(ServicesPresetOptions {
                relays: None,
                ..preset_options()
            })
            .is_ok(),
            "omitted relays must fall back to the n0 relays, as in Rust"
        );
        assert!(
            preset_iroh_services(ServicesPresetOptions {
                relays: Some(vec![]),
                ..preset_options()
            })
            .is_err(),
            "an explicitly empty list must error, not silently fall back"
        );
        assert!(
            preset_iroh_services(ServicesPresetOptions {
                relays: Some(vec!["not a url".to_string()]),
                ..preset_options()
            })
            .is_err(),
            "must reject a malformed relay url"
        );
    }

    #[tokio::test]
    async fn test_services_client_rejects_malformed_secret() {
        let ep = minimal_endpoint().await;
        let res = ServicesClient::create(
            &ep,
            ServicesOptions {
                api_secret: Some("not-a-valid-ticket".to_string()),
                ..Default::default()
            },
        )
        .await;
        assert!(res.is_err(), "must reject a malformed api secret");
        ep.close().await.unwrap();
    }
}
