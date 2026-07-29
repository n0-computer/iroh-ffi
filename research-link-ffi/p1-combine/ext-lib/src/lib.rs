//! Stand-in for `iroh-ping-ffi`: a separate crate, separate uniffi namespace,
//! that takes the *core* crate's object type as a parameter.

use std::sync::Arc;

use core_lib::Endpoint;

uniffi::setup_scaffolding!("ext_lib");

// No declaration needed for `core_lib::Endpoint`: `#[derive(uniffi::Object)]`
// emits `impl<UT> FfiConverter<UT>`, generic over the crate tag, so any other
// uniffi crate can use the type directly. Bindgen records the defining crate in
// the type's `module_path` and emits a cross-namespace reference.

#[derive(uniffi::Object)]
pub struct Ping {}

#[uniffi::export]
impl Ping {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }

    /// Reaches through the FFI wrapper into the *Rust* value. This is the thing
    /// that forces the two crates to agree on the compiled layout of `Inner`.
    pub fn ping(&self, endpoint: Arc<Endpoint>) -> String {
        format!(
            "ping via endpoint {} (core global @ {:#x})",
            endpoint.inner().id,
            endpoint.global_addr()
        )
    }
}
