use napi_derive::napi;

use crate::PublicKey;

#[napi]
impl PublicKey {
    /// String representation
    #[napi(js_name = "toString")]
    pub fn to_string_js(&self) -> String {
        self.to_string()
    }
}
