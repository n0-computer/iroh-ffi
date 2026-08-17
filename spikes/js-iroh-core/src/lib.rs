#![deny(clippy::all)]
use napi_derive::napi;

/// Forces the js-iroh dylib to be linked, so its `#[napi]` registrations run and Endpoint /
/// EndpointAddr appear in this addon's exports. Without a real reference the linker drops
/// the dylib entirely and nothing gets registered.
#[napi]
pub fn version() -> String {
    let _ = std::mem::size_of::<js_iroh::Endpoint>();
    env!("CARGO_PKG_VERSION").to_string()
}
