use napi_derive::napi;
use tracing_subscriber::filter::LevelFilter;

mod endpoint;
mod key;
mod net;
mod path;
mod relay;
mod services;
mod ticket;
mod watch;

pub use endpoint::*;
pub use key::*;
pub use net::*;
pub use path::*;
pub use relay::*;
pub use services::*;
pub use ticket::*;
pub use watch::*;

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
