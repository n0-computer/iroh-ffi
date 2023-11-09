use std::fmt::Display;

/// An Error.
#[derive(Debug, thiserror::Error)]
pub enum IrohError {
    #[error("runtime error: {description}")]
    Runtime { description: String },
    #[error("node creation failed: {description}")]
    NodeCreate { description: String },
    #[error("doc error: {description}")]
    Doc { description: String },
    #[error("author error: {description}")]
    Author { description: String },
    #[error("namespace error: {description}")]
    Namespace { description: String },
    #[error("doc ticket error: {description}")]
    DocTicket { description: String },
    #[error("uniffi: {description}")]
    Uniffi { description: String },
    #[error("connection: {description}")]
    Connection { description: String },
    #[error("blobs: {description}")]
    Blobs { description: String },
    #[error("Ipv4Addr error: {description}")]
    Ipv4Addr { description: String },
    #[error("SocketAddrV4 error: {description}")]
    SocketAddrV4 { description: String },
    #[error("Ipv6Addr error: {description}")]
    Ipv6Addr { description: String },
    #[error("SocketAddrV6 error: {description}")]
    SocketAddrV6 { description: String },
    #[error("PublicKey error: {description}")]
    PublicKey { description: String },
    #[error("NodeAddr error: {description}")]
    NodeAddr { description: String },
    #[error("Hash error: {description}")]
    Hash { description: String },
    #[error("RequestToken error: {description}")]
    RequestToken { description: String },
}

impl IrohError {
    pub fn runtime(error: impl Display) -> Self {
        IrohError::Runtime {
            description: error.to_string(),
        }
    }

    pub fn node_create(error: impl Display) -> Self {
        IrohError::NodeCreate {
            description: error.to_string(),
        }
    }

    pub fn author(error: impl Display) -> Self {
        IrohError::Author {
            description: error.to_string(),
        }
    }

    pub fn namespace(error: impl Display) -> Self {
        IrohError::Namespace {
            description: error.to_string(),
        }
    }

    pub fn connection(error: impl Display) -> Self {
        IrohError::Connection {
            description: error.to_string(),
        }
    }

    pub fn doc(error: impl Display) -> Self {
        IrohError::Doc {
            description: error.to_string(),
        }
    }

    pub fn doc_ticket(error: impl Display) -> Self {
        IrohError::DocTicket {
            description: error.to_string(),
        }
    }

    pub fn blobs(error: impl Display) -> Self {
        IrohError::Blobs {
            description: error.to_string(),
        }
    }

    pub fn ipv4_addr(error: impl Display) -> Self {
        IrohError::Ipv4Addr {
            description: error.to_string(),
        }
    }

    pub fn ipv6_addr(error: impl Display) -> Self {
        IrohError::Ipv6Addr {
            description: error.to_string(),
        }
    }

    pub fn socket_addr_v4(error: impl Display) -> Self {
        IrohError::SocketAddrV4 {
            description: error.to_string(),
        }
    }

    pub fn socket_addr_v6(error: impl Display) -> Self {
        IrohError::SocketAddrV6 {
            description: error.to_string(),
        }
    }

    pub fn public_key(error: impl Display) -> Self {
        IrohError::PublicKey {
            description: error.to_string(),
        }
    }

    pub fn node_addr(error: impl Display) -> Self {
        IrohError::NodeAddr {
            description: error.to_string(),
        }
    }

    pub fn hash(error: impl Display) -> Self {
        IrohError::Hash {
            description: error.to_string(),
        }
    }

    pub fn request_token(error: impl Display) -> Self {
        IrohError::RequestToken {
            description: error.to_string(),
        }
    }
}

impl From<uniffi::UnexpectedUniFFICallbackError> for IrohError {
    fn from(value: uniffi::UnexpectedUniFFICallbackError) -> Self {
        IrohError::Uniffi {
            description: value.to_string(),
        }
    }
}
