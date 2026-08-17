#![deny(clippy::all)]
use napi_derive::napi;

/// A class defined in the SHARED DYLIB, not in any addon.
#[napi]
pub struct SharedThing {
    value: u32,
}

#[napi]
impl SharedThing {
    #[napi(constructor)]
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    #[napi]
    pub fn value(&self) -> u32 {
        self.value
    }
}

#[napi]
pub fn shared_hello() -> String {
    "hello from the shared dylib".to_string()
}
