use std::time::Duration;

use iroh_services::{Client, ClientBuilder};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::Endpoint;

/// Build options for [`ServicesClient`].
#[derive(Debug, Default)]
#[napi(object)]
pub struct ServicesOptions {
    /// Encoded API secret string (`services1...`).
    pub api_secret: Option<String>,
    /// If true, read the API secret from `IROH_SERVICES_API_SECRET`.
    pub api_secret_from_env: Option<bool>,
    /// Unencrypted PEM-encoded OpenSSH ed25519 private key.
    pub ssh_key_pem: Option<String>,
    /// Optional endpoint name to register cloud-side.
    pub name: Option<String>,
    /// Metrics push interval (ms). `0` disables interval pushes.
    pub metrics_interval_ms: Option<i64>,
}

/// Flattened summary of an `iroh_services` diagnostics report.
#[derive(Debug, Clone)]
#[napi(object)]
pub struct DiagnosticsSummary {
    pub endpoint_id: String,
    pub direct_addrs: Vec<String>,
    pub iroh_version: String,
    pub iroh_services_version: String,
    pub has_net_report: bool,
    pub upnp: Option<bool>,
    pub pcp: Option<bool>,
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
#[napi]
pub struct ServicesClient {
    inner: Client,
}

#[napi]
impl ServicesClient {
    /// Build a new client bound to the given endpoint.
    #[napi(factory)]
    pub async fn create(endpoint: &Endpoint, options: ServicesOptions) -> Result<ServicesClient> {
        let mut builder: ClientBuilder = Client::builder(endpoint.raw());

        let creds = [
            options.api_secret.is_some(),
            options.api_secret_from_env.unwrap_or(false),
            options.ssh_key_pem.is_some(),
        ]
        .into_iter()
        .filter(|x| *x)
        .count();
        if creds != 1 {
            return Err(anyhow::anyhow!(
                "ServicesOptions requires exactly one of api_secret / api_secret_from_env / ssh_key_pem"
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
                builder = builder.metrics_interval(Duration::from_millis(ms as u64));
            }
        }

        let inner = builder
            .build()
            .await
            .map_err(|e| anyhow::anyhow!("services build failed: {e:?}"))?;
        Ok(ServicesClient { inner })
    }

    /// Read the current endpoint name from the local client.
    #[napi]
    pub async fn name(&self) -> Result<Option<String>> {
        self.inner
            .name()
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}").into())
    }

    /// Set the endpoint name cloud-side.
    #[napi]
    pub async fn set_name(&self, name: String) -> Result<()> {
        self.inner
            .set_name(name)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}").into())
    }

    /// Ping the remote service.
    #[napi]
    pub async fn ping(&self) -> Result<()> {
        self.inner
            .ping()
            .await
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("{e:?}").into())
    }

    /// Force a metrics flush.
    #[napi]
    pub async fn push_metrics(&self) -> Result<()> {
        self.inner
            .push_metrics()
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}").into())
    }

    /// Run a network-diagnostics report, optionally submitting it.
    #[napi]
    pub async fn submit_network_diagnostics(&self, send: bool) -> Result<DiagnosticsSummary> {
        let report = self
            .inner
            .net_diagnostics(send)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        Ok(report.into())
    }
}
