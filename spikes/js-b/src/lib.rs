#![deny(clippy::all)]
use napi_derive::napi;

/// THE LOAD-BEARING TEST: takes a `SharedThing` that addon A created.
///
/// napi resolves this by `napi_unwrap`ing the JS object to `*mut SharedThing` with no type
/// tag check. It is only sound if both addons see the SAME monomorphization of the type,
/// which the shared dylib is supposed to provide.
#[napi]
pub fn b_reads_shared(thing: &js_shared::SharedThing) -> u32 {
    thing.value() * 10
}

#[napi]
pub fn b_hello() -> String {
    "hello from addon B".to_string()
}
