use std::{str::FromStr, time::Duration};

use iroh::protocol::ProtocolHandler as _;
use iroh_services::{ApiSecret, Client, ClientBuilder, caps::NetDiagnosticsCap};
use n0_future::task::AbortOnDropHandle;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use tracing::warn;

use crate::{Connection, Endpoint};

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
    /// When true, let the iroh-services platform run network diagnostics
    /// against this endpoint on demand: `create` grants the net-diagnostics
    /// capability to the platform endpoint named in the API secret. Requires
    /// an api-secret credential. The endpoint must also serve the dial-back
    /// protocol; see `ClientHost`.
    ///
    /// The grant is best-effort: it runs in the background and is retried
    /// until it succeeds or the client is dropped; failures are logged, not
    /// surfaced. Each grant is valid for 30 days and re-issued on every
    /// client creation. A diagnostics run shares the endpoint's network
    /// details (direct addresses, NAT characteristics, relay latencies) with
    /// the platform.
    pub remote_diagnostics: Option<bool>,
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
    /// Owns the background capability-grant task spawned by `create` when
    /// `remote_diagnostics` is set; aborted when the client is dropped.
    _grant_task: Option<AbortOnDropHandle<()>>,
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

        // The grant target is the platform endpoint named in the API secret,
        // so remote diagnostics cannot work with an ssh key credential.
        let remote_diagnostics = options.remote_diagnostics.unwrap_or(false);
        if remote_diagnostics && options.ssh_key_pem.is_some() {
            return Err(anyhow::anyhow!(
                "remoteDiagnostics requires an apiSecret (or apiSecretFromEnv) credential"
            )
            .into());
        }

        // The platform endpoint id, extracted from the api secret before it
        // moves into the builder; this is who the capability grant targets.
        let mut platform_id: Option<iroh::EndpointId> = None;
        if let Some(secret) = options.api_secret {
            let ticket = ApiSecret::from_str(&secret)
                .map_err(|e| anyhow::anyhow!("invalid api secret: {e:?}"))?;
            platform_id = Some(ticket.addr().id);
            builder = builder
                .api_secret(ticket)
                .map_err(|e| anyhow::anyhow!("creating api token: {e:?}"))?;
        } else if options.api_secret_from_env.unwrap_or(false) {
            let ticket = ApiSecret::from_env_var(iroh_services::API_SECRET_ENV_VAR_NAME)
                .map_err(|e| anyhow::anyhow!("api secret env var: {e:?}"))?;
            platform_id = Some(ticket.addr().id);
            builder = builder
                .api_secret(ticket)
                .map_err(|e| anyhow::anyhow!("creating api token: {e:?}"))?;
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

        // The grant dials the (possibly offline) platform endpoint, so run it
        // in the background instead of blocking client startup on it. Retry
        // until it lands: the feature targets endpoints with bad networks,
        // which are exactly the ones likely to start offline. Grants are
        // idempotent and the task dies with the client.
        let grant_task = match (remote_diagnostics, platform_id) {
            (true, Some(platform_id)) => {
                let client = inner.clone();
                Some(AbortOnDropHandle::new(n0_future::task::spawn(async move {
                    let mut delay = Duration::from_secs(1);
                    while let Err(err) = client
                        .grant_capability(platform_id, [NetDiagnosticsCap::GetAny])
                        .await
                    {
                        warn!(
                            "failed to grant net-diagnostics capability, \
                             retrying in {delay:?}: {err:?}"
                        );
                        tokio::time::sleep(delay).await;
                        delay = (delay * 2).min(Duration::from_secs(60));
                    }
                })))
            }
            _ => None,
        };

        Ok(ServicesClient {
            inner,
            _grant_task: grant_task,
        })
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

/// The ALPN of the iroh-services dial-back protocol served by `ClientHost`.
#[napi]
pub fn client_host_alpn() -> Vec<u8> {
    iroh_services::CLIENT_HOST_ALPN.to_vec()
}

/// Serves the iroh-services dial-back protocol on an endpoint.
///
/// The platform connects on `clientHostAlpn()` to run network diagnostics on
/// demand (the dashboard's Run Diagnostics button). Incoming requests must
/// present a capability token issued by this endpoint; pair with
/// `ServicesOptions.remoteDiagnostics`, which grants that token to the
/// platform.
///
/// Construct one `ClientHost` and, in the accept loop, call
/// `handleConnection` for each connection whose ALPN equals
/// `clientHostAlpn()`.
///
/// Like any served protocol, the ALPN is an open accept surface: anyone can
/// connect, but requests without a capability token issued by this endpoint
/// are rejected before diagnostics run.
#[napi]
pub struct ClientHost {
    inner: iroh_services::ClientHost,
}

#[napi]
impl ClientHost {
    /// Create a host serving diagnostics for the given endpoint.
    #[napi(constructor)]
    pub fn new(endpoint: &Endpoint) -> Self {
        Self {
            inner: iroh_services::ClientHost::new(endpoint.raw()),
        }
    }

    /// Serve one accepted dial-back connection to completion.
    ///
    /// The returned promise resolves only once the remote closes the
    /// connection, which can take tens of seconds while diagnostics run. Do
    /// not await it inline in an accept loop; let it run concurrently
    /// (`host.handleConnection(conn).catch(...)`) so the loop keeps
    /// accepting.
    #[napi]
    pub async fn handle_connection(&self, conn: &Connection) -> Result<()> {
        self.inner
            .accept(conn.raw().clone())
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}").into())
    }
}
