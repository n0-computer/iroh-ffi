mod accept;
mod endpoint;
mod error;
mod key;
mod net;
mod path;
mod relay;
mod services;
mod ticket;
mod watch;

use tracing_subscriber::filter::LevelFilter;

pub use self::{
    accept::*, endpoint::*, error::*, key::*, net::*, path::*, relay::*, services::*, ticket::*,
    watch::*,
};

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
