mod author;
mod blob;
mod doc;
mod key;
mod node;
mod tag;

pub use self::blob::*;
pub use self::doc::*;
pub use self::node::*;
pub use self::tag::*;

use crate::{key_to_path, path_to_key, IrohError};
use napi::bindgen_prelude::BigInt;
use napi_derive::napi;

impl From<IrohError> for napi::JsError {
    fn from(value: IrohError) -> Self {
        anyhow::Error::from(value).into()
    }
}

impl From<IrohError> for napi::Error {
    fn from(value: IrohError) -> Self {
        napi::Error::new(napi::Status::GenericFailure, value.to_string())
    }
}

/// Helper function that translates a key that was derived from the [`path_to_key`] function back
/// into a path.
///
/// If `prefix` exists, it will be stripped before converting back to a path
/// If `root` exists, will add the root as a parent to the created path
/// Removes any null byte that has been appened to the key
#[napi(js_name = "keyToPath")]
pub fn key_to_path_js(
    key: napi::bindgen_prelude::Buffer,
    prefix: Option<String>,
    root: Option<String>,
) -> Result<String, IrohError> {
    let key: Vec<_> = key.into();
    key_to_path(key, prefix, root)
}

/// Helper function that creates a document key from a canonicalized path, removing the `root` and adding the `prefix`, if they exist
///
/// Appends the null byte to the end of the key.
#[napi(js_name = "pathToKey")]
pub fn path_to_key_js(
    path: String,
    prefix: Option<String>,
    root: Option<String>,
) -> Result<napi::bindgen_prelude::Buffer, IrohError> {
    let key = path_to_key(path, prefix, root)?;
    Ok(key.into())
}

fn u64_from_bigint(i: BigInt) -> napi::Result<u64> {
    let (signed, num, lossless) = i.get_u64();
    if signed {
        return Err(napi::Error::new(
            napi::Status::GenericFailure,
            "conversion error: expected unsigned integer",
        ));
    }
    if !lossless {
        return Err(napi::Error::new(
            napi::Status::GenericFailure,
            "conversion error: overflow",
        ));
    }
    Ok(num)
}
