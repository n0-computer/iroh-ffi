use napi_derive::napi;
use p4_napi_core::Endpoint;

/// Stand-in for an `iroh-ping` napi addon: a *second* `.node` file that accepts
/// a class instance constructed by the first one.
#[napi]
pub struct Ping {}

#[napi]
impl Ping {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {}
    }

    /// Takes the core addon's class by reference and reaches into the Rust value.
    #[napi]
    pub fn ping(&self, endpoint: &Endpoint) -> String {
        format!(
            "ping via endpoint {} (core global @ {})",
            endpoint.inner().id,
            endpoint.global_addr()
        )
    }
}
