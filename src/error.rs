/// Stable high-level error categories exposed across the FFI boundary.
///
/// These are intentionally coarser than the upstream Rust error types. They
/// give foreign bindings a stable taxonomy for `errors.Is`-style handling
/// without leaking the internal `iroh` / `n0-error` error hierarchy.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum IrohErrorKind {
    /// Invalid input supplied by the caller.
    InvalidInput,
    /// Failure while binding an endpoint.
    Bind,
    /// Failure while initiating or completing an outgoing connection.
    Connect,
    /// An established connection failed or closed unexpectedly.
    Connection,
    /// ALPN negotiation or lookup failed.
    Alpn,
    /// Endpoint id / public key parsing failed.
    KeyParsing,
    /// Ticket parsing failed.
    TicketParsing,
    /// Relay configuration or relay operation failed.
    Relay,
    /// Stream read/write/control operation failed.
    Stream,
    /// Datagram send/receive operation failed.
    Datagram,
    /// Foreign callback failed.
    Callback,
    /// Operation was attempted on a closed stream/connection/resource.
    Closed,
    /// Operation timed out.
    Timeout,
    /// Unclassified internal error.
    Internal,
}

/// An Error.
#[derive(Debug, thiserror::Error, uniffi::Object)]
#[error("{message}")]
#[uniffi::export(Debug)]
pub struct IrohError {
    kind: IrohErrorKind,
    message: String,
    debug_message: String,
}

impl IrohError {
    pub(crate) fn new(
        kind: IrohErrorKind,
        message: impl Into<String>,
        debug_message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            debug_message: debug_message.into(),
        }
    }

    pub(crate) fn invalid_input(message: impl Into<String>) -> Self {
        let message = message.into();
        Self::new(IrohErrorKind::InvalidInput, message.clone(), message)
    }

    fn from_debug(kind: IrohErrorKind, value: impl std::fmt::Debug) -> Self {
        let debug_message = format!("{value:?}");
        Self::new(kind, debug_message.clone(), debug_message)
    }
}

#[uniffi::export]
impl IrohError {
    /// Human-readable error message.
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// Stable high-level error category.
    pub fn kind(&self) -> IrohErrorKind {
        self.kind
    }

    /// Detailed debug representation of the original Rust error.
    pub fn debug_message(&self) -> String {
        self.debug_message.clone()
    }

    /// Convenience helper for bindings that do not expose enum comparison
    /// ergonomically.
    pub fn is_kind(&self, kind: IrohErrorKind) -> bool {
        self.kind == kind
    }
}

impl From<anyhow::Error> for IrohError {
    fn from(e: anyhow::Error) -> Self {
        let message = e.to_string();
        let debug_message = format!("{e:?}");
        Self::new(IrohErrorKind::Internal, message, debug_message)
    }
}

/// Conversion helper for upstream iroh / n0-error typed errors. The variants
/// below deliberately map to a stable FFI-level taxonomy rather than exposing
/// upstream Rust error types directly.
macro_rules! from_iroh_err {
    ($($path:path => $kind:expr),* $(,)?) => {
        $(
            impl From<$path> for IrohError {
                fn from(value: $path) -> Self {
                    Self::from_debug($kind, value)
                }
            }
        )*
    };
}

from_iroh_err! {
    iroh::endpoint::BindError => IrohErrorKind::Bind,
    iroh::endpoint::ConnectError => IrohErrorKind::Connect,
    iroh::endpoint::ConnectionError => IrohErrorKind::Connection,
    iroh::endpoint::AlpnError => IrohErrorKind::Alpn,
    iroh::endpoint::RemoteEndpointIdError => IrohErrorKind::InvalidInput,
    iroh::endpoint::VarIntBoundsExceeded => IrohErrorKind::InvalidInput,
    iroh::endpoint::WriteError => IrohErrorKind::Stream,
    iroh::endpoint::ClosedStream => IrohErrorKind::Closed,
    iroh::endpoint::ReadError => IrohErrorKind::Stream,
    iroh::endpoint::ReadExactError => IrohErrorKind::Stream,
    iroh::endpoint::ReadToEndError => IrohErrorKind::Stream,
    iroh::endpoint::StoppedError => IrohErrorKind::Stream,
    iroh::endpoint::SendDatagramError => IrohErrorKind::Datagram,
    iroh::endpoint::ResetError => IrohErrorKind::Stream,
    iroh_base::KeyParsingError => IrohErrorKind::KeyParsing,
    iroh_tickets::ParseError => IrohErrorKind::TicketParsing,
    n0_future::task::JoinError => IrohErrorKind::Internal,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq, uniffi::Error)]
pub enum CallbackError {
    #[error("Callback failed")]
    Error,
}

impl From<CallbackError> for IrohError {
    fn from(e: CallbackError) -> Self {
        IrohError::new(IrohErrorKind::Callback, e.to_string(), format!("{e:?}"))
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
