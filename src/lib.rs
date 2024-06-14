mod author;
mod blob;
mod doc;
mod error;
mod key;
mod node;
mod tag;
mod ticket;

pub use self::author::*;
pub use self::blob::*;
pub use self::doc::*;
pub use self::error::*;
pub use self::key::*;
pub use self::node::*;
pub use self::tag::*;
pub use self::ticket::*;

use futures::Future;
use iroh::metrics::try_init_metrics_collection;

use tracing_subscriber::filter::LevelFilter;

// This macro includes the scaffolding for the Iroh FFI bindings.
uniffi::include_scaffolding!("iroh");

/// The logging level. See the rust (log crate)[https://docs.rs/log] for more information.
#[derive(Debug)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> LevelFilter {
        match level {
            LogLevel::Trace => LevelFilter::TRACE,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Off => LevelFilter::OFF,
        }
    }
}

/// Set the logging level.
pub fn set_log_level(level: LogLevel) {
    use tracing_subscriber::{fmt, prelude::*, reload};
    let filter: LevelFilter = level.into();
    let (filter, _) = reload::Layer::new(filter);
    let mut layer = fmt::Layer::default();
    layer.set_ansi(false);
    tracing_subscriber::registry()
        .with(filter)
        .with(layer)
        .init();
}

/// Initialize the global metrics collection.
pub fn start_metrics_collection() -> Result<(), IrohError> {
    try_init_metrics_collection().map_err(|e| anyhow::Error::from(e).into())
}

fn block_on<F: Future<Output = T>, T>(rt: &tokio::runtime::Handle, fut: F) -> T {
    tokio::task::block_in_place(move || match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle.block_on(fut),
        Err(_) => rt.block_on(fut),
    })
}

/// Helper function that translates a key that was derived from the [`path_to_key`] function back
/// into a path.
///
/// If `prefix` exists, it will be stripped before converting back to a path
/// If `root` exists, will add the root as a parent to the created path
/// Removes any null byte that has been appened to the key
pub fn key_to_path(
    key: Vec<u8>,
    prefix: Option<String>,
    root: Option<String>,
) -> Result<String, IrohError> {
    let path = iroh::util::fs::key_to_path(key, prefix, root.map(std::path::PathBuf::from))?;
    let path = path.to_str();
    let path = path.ok_or_else(|| anyhow::anyhow!("Unable to parse path {:?}", path))?;
    let path = path.to_string();
    Ok(path)
}

/// Helper function that creates a document key from a canonicalized path, removing the `root` and adding the `prefix`, if they exist
///
/// Appends the null byte to the end of the key.
pub fn path_to_key(
    path: String,
    prefix: Option<String>,
    root: Option<String>,
) -> Result<Vec<u8>, IrohError> {
    iroh::util::fs::path_to_key(
        std::path::PathBuf::from(path),
        prefix,
        root.map(std::path::PathBuf::from),
    )
    .map(|k| k.to_vec())
    .map_err(IrohError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_path_to_key_roundtrip() {
        let path = std::path::PathBuf::from("/").join("foo").join("bar");
        let path = path.to_str().unwrap().to_string();
        let mut key = b"/foo/bar\0".to_vec();

        let got_key = path_to_key(path.clone(), None, None).unwrap();
        assert_eq!(key, got_key);
        let got_path = key_to_path(got_key.clone(), None, None).unwrap();
        assert_eq!(path, got_path);

        // including prefix
        let prefix = String::from("prefix:");
        key = b"prefix:/foo/bar\0".to_vec();

        let got_key = path_to_key(path.clone(), Some(prefix.clone()), None).unwrap();
        assert_eq!(key, got_key);
        let got_path = key_to_path(got_key.clone(), Some(prefix.clone()), None).unwrap();
        assert_eq!(path, got_path);

        // including root
        let root = std::path::PathBuf::from("/").join("foo");
        let root = root.to_str().unwrap().to_string();
        key = b"prefix:bar\0".to_vec();

        let got_key = path_to_key(path.clone(), Some(prefix.clone()), Some(root.clone())).unwrap();
        assert_eq!(key, got_key);
        let got_path =
            key_to_path(got_key.clone(), Some(prefix.clone()), Some(root.clone())).unwrap();
        assert_eq!(path, got_path);
    }
}
