//! Binding for `iroh-services` — push metrics to services.iroh.computer and
//! serve platform-initiated network diagnostics.
//!
//! Mirrors `iroh_services::Client`. Construct via [`ServicesClient::create`] with
//! a built [`Endpoint`] plus credentials supplied through [`ServicesOptions`].
//! [`ClientHost`] and [`services_client_host`] serve the dial-back protocol the
//! platform uses to run diagnostics on demand.

use std::{str::FromStr, sync::Arc, time::Duration};

use iroh::protocol::ProtocolHandler as _;
use iroh_services::{ApiSecret, Client, ClientBuilder, caps::NetDiagnosticsCap};
use n0_future::task::AbortOnDropHandle;
use tracing::warn;

use crate::{CallbackError, Connection, Endpoint, IrohError, ProtocolCreator, ProtocolHandler};

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
    /// When true, let the iroh-services platform run network diagnostics
    /// against this endpoint on demand: [`ServicesClient::create`] grants the
    /// net-diagnostics capability to the platform endpoint named in the API
    /// secret. Requires an api-secret credential. The endpoint must also
    /// serve the dial-back protocol; see [`ClientHost`] and
    /// [`services_client_host`].
    ///
    /// The grant is best-effort: it runs in the background and is retried
    /// until it succeeds or the client is dropped; failures are logged, not
    /// surfaced. Each grant is valid for 30 days and re-issued on every
    /// client creation. A diagnostics run shares the endpoint's network
    /// details (direct addresses, NAT characteristics, relay latencies) with
    /// the platform.
    #[uniffi(default = None)]
    pub remote_diagnostics: Option<bool>,
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
    /// Owns the background capability-grant task spawned by [`Self::create`]
    /// when `remote_diagnostics` is set; aborted when the last clone drops.
    _grant_task: Option<Arc<AbortOnDropHandle<()>>>,
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

        // The grant target is the platform endpoint named in the API secret,
        // so remote diagnostics cannot work with an ssh key credential.
        let remote_diagnostics = options.remote_diagnostics.unwrap_or(false);
        if remote_diagnostics && options.ssh_key_pem.is_some() {
            return Err(anyhow::anyhow!(
                "remote_diagnostics requires an api_secret (or api_secret_from_env) credential"
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
                builder = builder.metrics_interval(Duration::from_millis(ms));
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
                Some(Arc::new(AbortOnDropHandle::new(n0_future::task::spawn(
                    async move {
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
                    },
                ))))
            }
            _ => None,
        };

        Ok(ServicesClient {
            inner,
            _grant_task: grant_task,
        })
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

/// The ALPN of the iroh-services dial-back protocol served by [`ClientHost`].
#[uniffi::export]
pub fn client_host_alpn() -> Vec<u8> {
    iroh_services::CLIENT_HOST_ALPN.to_vec()
}

/// Serves the iroh-services dial-back protocol on an endpoint.
///
/// The platform connects on [`client_host_alpn`] to run network diagnostics
/// on demand (the dashboard's Run Diagnostics button). Incoming requests must
/// present a capability token issued by this endpoint; pair with
/// [`ServicesOptions::remote_diagnostics`], which grants that token to the
/// platform.
///
/// Apps that register protocol handlers at bind time should mount
/// [`services_client_host`] instead and never need this type. Apps that
/// drive their own accept loop construct a `ClientHost` once and call
/// [`Self::handle_connection`] for each connection whose ALPN equals
/// [`client_host_alpn`].
///
/// Like any served protocol, the ALPN is an open accept surface: anyone can
/// connect, but requests without a capability token issued by this endpoint
/// are rejected before diagnostics run.
#[derive(uniffi::Object)]
pub struct ClientHost {
    inner: iroh_services::ClientHost,
}

#[uniffi::export]
impl ClientHost {
    /// Create a host serving diagnostics for the given endpoint.
    #[uniffi::constructor]
    pub fn new(endpoint: &Endpoint) -> Self {
        Self {
            inner: iroh_services::ClientHost::new(endpoint.raw()),
        }
    }

    /// Serve one accepted dial-back connection to completion.
    ///
    /// The returned future resolves only once the remote closes the
    /// connection, which can take tens of seconds while diagnostics run. Do
    /// not await it inline in an accept loop; run it as its own task so the
    /// loop keeps accepting.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn handle_connection(&self, conn: &Connection) -> Result<(), IrohError> {
        self.inner
            .accept(conn.raw().clone())
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}").into())
    }
}

/// A [`ProtocolCreator`] serving the iroh-services dial-back protocol.
///
/// Mount it at bind time under [`client_host_alpn`] via
/// [`EndpointOptions::protocols`].
///
/// [`EndpointOptions::protocols`]: crate::EndpointOptions::protocols
#[uniffi::export]
pub fn services_client_host() -> Arc<dyn ProtocolCreator> {
    Arc::new(ClientHostCreator)
}

#[derive(Debug)]
struct ClientHostCreator;

impl ProtocolCreator for ClientHostCreator {
    fn create(&self, endpoint: Arc<Endpoint>) -> Arc<dyn ProtocolHandler> {
        Arc::new(ClientHostHandler {
            inner: iroh_services::ClientHost::new(endpoint.raw()),
        })
    }
}

struct ClientHostHandler {
    inner: iroh_services::ClientHost,
}

#[async_trait::async_trait]
impl ProtocolHandler for ClientHostHandler {
    async fn accept(&self, conn: Arc<Connection>) -> Result<(), CallbackError> {
        self.inner
            .accept(conn.raw().clone())
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}").into())
    }

    async fn shutdown(&self) {}
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

    #[tokio::test]
    async fn test_remote_diagnostics_rejects_ssh_key_credential() {
        let ep = minimal_endpoint().await;
        let res = ServicesClient::create(
            &ep,
            ServicesOptions {
                ssh_key_pem: Some("irrelevant".to_string()),
                remote_diagnostics: Some(true),
                ..Default::default()
            },
        )
        .await;
        // Assert on the message: a malformed pem also errors, and this test
        // must fail if the remote_diagnostics guard (not pem parsing) goes.
        let err = res
            .err()
            .expect("must reject ssh key credential")
            .to_string();
        assert!(
            err.contains("remote_diagnostics"),
            "expected the remote_diagnostics guard, got: {err}"
        );
        ep.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_remote_diagnostics_boots_with_fake_secret() {
        let ep = minimal_endpoint().await;
        let client = ServicesClient::create(
            &ep,
            ServicesOptions {
                api_secret: Some(FAKE_API_SECRET.to_string()),
                remote_diagnostics: Some(true),
                ..Default::default()
            },
        )
        .await
        .expect("client should construct; the grant runs (and fails) in the background");
        drop(client);
        ep.close().await.unwrap();
    }

    /// Full dial-back round trip against an FFI endpoint: mount the protocol
    /// via [`services_client_host`] at bind, mint a token with the same
    /// issuer/audience shape `grant_capability` delivers (issued by the
    /// server, audience the dialer), then authenticate and request a
    /// diagnostics report over irpc exactly like the platform would.
    ///
    /// Slow by construction (about 20s): the server runs a real diagnostics
    /// probe whose relay and net-report phases time out offline. There is no
    /// FFI knob to shorten the upstream timeouts.
    #[tokio::test]
    async fn test_client_host_serves_diagnostics() {
        use std::collections::HashMap;

        use iroh_services::{
            ClientHostClient,
            caps::{Caps, create_grant_token},
            protocol::{Auth, RunNetworkDiagnostics},
        };
        use irpc_iroh::IrohLazyRemoteConnection;

        let mut protocols: HashMap<Vec<u8>, Arc<dyn ProtocolCreator>> = HashMap::new();
        protocols.insert(client_host_alpn(), services_client_host());
        let server = Endpoint::bind(EndpointOptions {
            preset: Some(crate::preset_minimal()),
            protocols: Some(protocols),
            ..Default::default()
        })
        .await
        .unwrap();

        let dialer = iroh::Endpoint::builder(iroh::endpoint::presets::Minimal)
            .bind()
            .await
            .unwrap();

        let rcan = create_grant_token(
            server.raw().secret_key().clone(),
            dialer.id(),
            Duration::from_secs(60),
            Caps::new([NetDiagnosticsCap::GetAny]),
        )
        .unwrap();

        let conn =
            IrohLazyRemoteConnection::new(dialer.clone(), server.raw().addr(), client_host_alpn());
        let client = ClientHostClient::boxed(conn);

        client.rpc(Auth { caps: rcan }).await.unwrap();
        let report = client
            .rpc(RunNetworkDiagnostics)
            .await
            .unwrap()
            .expect("expected Ok(DiagnosticsReport)");
        assert_eq!(report.endpoint_id, server.raw().id());

        server.close().await.unwrap();
        dialer.close().await;
    }

    /// The manual accept-loop path: [`ClientHost::handle_connection`] must
    /// return once the remote goes away instead of hanging the loop forever.
    /// (The full protocol exchange is covered by the round-trip test above;
    /// this exercises the wiring JS and accept-loop apps use.)
    #[tokio::test]
    async fn test_handle_connection_returns_on_remote_close() {
        let server = Endpoint::bind(EndpointOptions {
            preset: Some(crate::preset_minimal()),
            alpns: Some(vec![client_host_alpn()]),
            ..Default::default()
        })
        .await
        .unwrap();
        let host = ClientHost::new(&server);

        let dialer = iroh::Endpoint::builder(iroh::endpoint::presets::Minimal)
            .bind()
            .await
            .unwrap();
        let server_addr = server.raw().addr();
        let dial = tokio::spawn(async move {
            let conn = dialer
                .connect(server_addr, iroh_services::CLIENT_HOST_ALPN)
                .await
                .unwrap();
            conn.close(0u32.into(), b"done");
            dialer.close().await;
        });

        let incoming = server.accept_next().await.unwrap();
        let accepting = incoming.accept().await.unwrap();
        let conn = accepting.connect().await.unwrap();
        // Ok or Err both fine; the handler just must not hang once the
        // remote has closed.
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            host.handle_connection(&conn),
        )
        .await
        .expect("handle_connection must return after the remote closes");

        dial.await.unwrap();
        server.close().await.unwrap();
    }
}
