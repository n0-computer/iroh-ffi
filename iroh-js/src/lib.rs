use iroh::metrics::try_init_metrics_collection;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use tracing_subscriber::filter::LevelFilter;

mod author;
mod blob;
mod doc;
mod endpoint;
mod gossip;
mod key;
mod net;
mod node;
mod ticket;

pub use author::*;
pub use blob::*;
pub use doc::*;
pub use endpoint::*;
pub use gossip::*;
pub use key::*;
pub use net::*;
pub use node::*;
pub use ticket::*;

/// The logging level. See the rust (log crate)[https://docs.rs/log] for more information.
#[derive(Debug)]
#[napi(string_enum)]
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
#[napi]
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
#[napi]
pub fn start_metrics_collection() -> Result<()> {
    try_init_metrics_collection()?;
    Ok(())
}

/// Helper function that translates a key that was derived from the [`path_to_key`] function back
/// into a path.
///
/// If `prefix` exists, it will be stripped before converting back to a path
/// If `root` exists, will add the root as a parent to the created path
/// Removes any null byte that has been appened to the key
#[napi]
pub fn key_to_path(key: Vec<u8>, prefix: Option<String>, root: Option<String>) -> Result<String> {
    let path = iroh::util::fs::key_to_path(key, prefix, root.map(std::path::PathBuf::from))?;
    let path = path.to_str();
    let path = path.ok_or_else(|| anyhow::anyhow!("Unable to parse path {:?}", path))?;
    let path = path.to_string();

    Ok(path)
}

/// Helper function that creates a document key from a canonicalized path, removing the `root` and adding the `prefix`, if they exist
///
/// Appends the null byte to the end of the key.
#[napi]
pub fn path_to_key(path: String, prefix: Option<String>, root: Option<String>) -> Result<Vec<u8>> {
    let key = iroh::util::fs::path_to_key(
        std::path::PathBuf::from(path),
        prefix,
        root.map(std::path::PathBuf::from),
    )?;

    Ok(key.to_vec())
}
