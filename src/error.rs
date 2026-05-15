/// An Error.
#[derive(Debug, thiserror::Error, uniffi::Object)]
#[error("{e:?}")]
#[uniffi::export(Debug)]
pub struct IrohError {
    e: anyhow::Error,
}

#[uniffi::export]
impl IrohError {
    pub fn message(&self) -> String {
        self.to_string()
    }
}

impl From<anyhow::Error> for IrohError {
    fn from(e: anyhow::Error) -> Self {
        Self { e }
    }
}

/// Catch-all conversion for the n0-error / snafu typed errors that iroh now
/// returns from its public APIs. Wraps them in an `anyhow::Error` via Debug
/// so we get the full stack trace in the FFI message.
macro_rules! from_iroh_err {
    ($($path:path),* $(,)?) => {
        $(
            impl From<$path> for IrohError {
                fn from(value: $path) -> Self {
                    Self {
                        e: anyhow::anyhow!("{:?}", value),
                    }
                }
            }
        )*
    };
}

from_iroh_err! {
    iroh::endpoint::BindError,
    iroh::endpoint::ConnectError,
    iroh::endpoint::ConnectionError,
    iroh::endpoint::AlpnError,
    iroh::endpoint::RemoteEndpointIdError,
    iroh::endpoint::VarIntBoundsExceeded,
    iroh::endpoint::WriteError,
    iroh::endpoint::ClosedStream,
    iroh::endpoint::ReadError,
    iroh::endpoint::ReadExactError,
    iroh::endpoint::ReadToEndError,
    iroh::endpoint::StoppedError,
    iroh::endpoint::SendDatagramError,
    iroh::endpoint::ResetError,
    iroh_base::KeyParsingError,
    iroh_tickets::ParseError,
    n0_future::task::JoinError,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq, uniffi::Error)]
pub enum CallbackError {
    #[error("Callback failed")]
    Error,
}

impl From<CallbackError> for IrohError {
    fn from(e: CallbackError) -> Self {
        IrohError {
            e: anyhow::anyhow!("{:?}", e),
        }
    }
}

impl From<anyhow::Error> for CallbackError {
    fn from(_e: anyhow::Error) -> Self {
        CallbackError::Error
    }
}

impl From<uniffi::UnexpectedUniFFICallbackError> for CallbackError {
    fn from(_: uniffi::UnexpectedUniFFICallbackError) -> Self {
        CallbackError::Error
    }
}
