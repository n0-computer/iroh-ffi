#![deny(clippy::all)]
use napi_derive::napi;

#[napi]
pub fn a_hello() -> String {
    "hello from addon A".to_string()
}

/// Forces the js-shared dylib to actually be linked, so its `#[napi]` ctor
/// registrations have a chance to run.
#[napi]
pub fn a_uses_shared() -> u32 {
    js_shared::SharedThing::new(7).value()
}
