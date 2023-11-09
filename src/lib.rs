mod blob;
mod doc;
mod error;
mod key;
mod net;
mod node;
mod tag;

pub use self::blob::*;
pub use self::doc::*;
pub use self::error::IrohError;
pub use self::key::*;
pub use self::net::*;
pub use self::node::*;
pub use self::tag::*;

use futures::Future;
use iroh::{bytes::util::runtime::Handle, metrics::try_init_metrics_collection};

use tracing_subscriber::filter::LevelFilter;

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

pub fn start_metrics_collection() -> Result<(), IrohError> {
    try_init_metrics_collection().map_err(IrohError::runtime)
}

fn block_on<F: Future<Output = T>, T>(rt: &Handle, fut: F) -> T {
    tokio::task::block_in_place(move || match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle.block_on(fut),
        Err(_) => rt.main().block_on(fut),
    })
}

uniffi::include_scaffolding!("iroh");
