mod author;
mod blob;
mod doc;
mod endpoint;
mod error;
mod gossip;
mod key;
mod net;
mod node;
mod tag;
mod ticket;

use std::path::{Component, Path, PathBuf};

use bytes::Bytes;
use tracing_subscriber::filter::LevelFilter;

pub use self::{
    author::*, blob::*, doc::*, endpoint::*, error::*, gossip::*, key::*, net::*, node::*, tag::*,
    ticket::*,
};

// This macro includes the scaffolding for the Iroh FFI bindings.
uniffi::setup_scaffolding!();

/// The logging level. See the rust (log crate)[https://docs.rs/log] for more information.
#[derive(Debug, uniffi::Enum)]
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
#[uniffi::export]
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

/// Helper function that translates a key that was derived from the [`path_to_key`] function back
/// into a path.
///
/// If `prefix` exists, it will be stripped before converting back to a path
/// If `root` exists, will add the root as a parent to the created path
/// Removes any null byte that has been appened to the key
#[uniffi::export]
pub fn key_to_path(
    key: Vec<u8>,
    prefix: Option<String>,
    root: Option<String>,
) -> Result<String, IrohError> {
    let path = inner_key_to_path(key, prefix, root.map(std::path::PathBuf::from))?;
    let path = path.to_str();
    let path = path.ok_or_else(|| anyhow::anyhow!("Unable to parse path {:?}", path))?;
    let path = path.to_string();
    Ok(path)
}

fn inner_key_to_path(
    key: impl AsRef<[u8]>,
    prefix: Option<String>,
    root: Option<PathBuf>,
) -> anyhow::Result<PathBuf> {
    let mut key = key.as_ref();
    if key.is_empty() {
        return Ok(PathBuf::new());
    }
    // if the last element is the null byte, remove it
    if b'\0' == key[key.len() - 1] {
        key = &key[..key.len() - 1]
    }

    let key = if let Some(prefix) = prefix {
        let prefix = prefix.into_bytes();
        if prefix[..] == key[..prefix.len()] {
            &key[prefix.len()..]
        } else {
            anyhow::bail!("key {:?} does not begin with prefix {:?}", key, prefix);
        }
    } else {
        key
    };

    let mut path = if key[0] == b'/' {
        PathBuf::from("/")
    } else {
        PathBuf::new()
    };
    for component in key
        .split(|c| c == &b'/')
        .map(|c| String::from_utf8(c.into()).context("key contains invalid data"))
    {
        let component = component?;
        path = path.join(component);
    }

    // add root if it exists
    let path = if let Some(root) = root {
        root.join(path)
    } else {
        path
    };

    Ok(path)
}

/// Helper function that creates a document key from a canonicalized path, removing the `root` and adding the `prefix`, if they exist
///
/// Appends the null byte to the end of the key.
#[uniffi::export]
pub fn path_to_key(
    path: String,
    prefix: Option<String>,
    root: Option<String>,
) -> Result<Vec<u8>, IrohError> {
    inner_path_to_key(
        std::path::PathBuf::from(path),
        prefix,
        root.map(std::path::PathBuf::from),
    )
    .map(|k| k.to_vec())
    .map_err(IrohError::from)
}

fn inner_path_to_key(
    path: impl AsRef<Path>,
    prefix: Option<String>,
    root: Option<PathBuf>,
) -> anyhow::Result<Bytes> {
    let path = path.as_ref();
    let path = if let Some(root) = root {
        path.strip_prefix(root)?
    } else {
        path
    };
    let suffix = canonicalized_path_to_string(path, false)?.into_bytes();
    let mut key = if let Some(prefix) = prefix {
        prefix.into_bytes().to_vec()
    } else {
        Vec::new()
    };
    key.extend(suffix);
    key.push(b'\0');
    Ok(key.into())
}
fn canonicalized_path_to_string(
    path: impl AsRef<Path>,
    must_be_relative: bool,
) -> anyhow::Result<String> {
    let mut path_str = String::new();
    let parts = path
        .as_ref()
        .components()
        .filter_map(|c| match c {
            Component::Normal(x) => {
                let c = match x.to_str() {
                    Some(c) => c,
                    None => return Some(Err(anyhow::anyhow!("invalid character in path"))),
                };

                if !c.contains('/') && !c.contains('\\') {
                    Some(Ok(c))
                } else {
                    Some(Err(anyhow::anyhow!("invalid path component {:?}", c)))
                }
            }
            Component::RootDir => {
                if must_be_relative {
                    Some(Err(anyhow::anyhow!("invalid path component {:?}", c)))
                } else {
                    path_str.push('/');
                    None
                }
            }
            _ => Some(Err(anyhow::anyhow!("invalid path component {:?}", c))),
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let parts = parts.join("/");
    path_str.push_str(&parts);
    Ok(path_str)
}

#[cfg(test)]
fn setup_logging() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .try_init()
        .ok();
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
