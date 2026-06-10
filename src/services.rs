//! Binding for `iroh-services` — push metrics to services.iroh.computer.
//!
//! Mirrors `iroh_services::Client`. Construct via [`ServicesClient::create`] with
//! a built [`Endpoint`] plus credentials supplied through [`ServicesOptions`].

use std::time::Duration;

use iroh_services::{Client, ClientBuilder};

use crate::{Endpoint, IrohError};

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
