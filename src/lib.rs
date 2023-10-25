mod error;
mod key;
mod net;
mod node;

pub use self::error::IrohError;
pub use self::key::*;
pub use self::net::*;
pub use self::node::*;

use tracing_subscriber::filter::LevelFilter;

uniffi::include_scaffolding!("iroh");

use iroh::metrics::try_init_metrics_collection;

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
    try_init_metrics_collection().map_err(|e| IrohError::Runtime {
        description: e.to_string(),
    })?;
    Ok(())
}
