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
    #[error("blob ticket error: {description}")]
    BlobTicket { description: String },
    #[error("doc ticket error: {description}")]
    DocTicket { description: String },
    #[error("uniffi: {description}")]
    Uniffi { description: String },
    #[error("connection: {description}")]
    Connection { description: String },
    #[error("blobs: {description}")]
    Blobs { description: String },
    #[error("collection: {description}")]
    Collection { description: String },
    #[error("Ipv4Addr error: {description}")]
    Ipv4Addr { description: String },
    #[error("Ipv6Addr error: {description}")]
    Ipv6Addr { description: String },
    #[error("SocketAddr error: {description}")]
    SocketAddr { description: String },
    #[error("PublicKey error: {description}")]
    PublicKey { description: String },
    #[error("NodeAddr error: {description}")]
    NodeAddr { description: String },
    #[error("Hash error: {description}")]
    Hash { description: String },
    #[error("FsUtil error: {description}")]
    FsUtil { description: String },
    #[error("Tags error: {description}")]
    Tags { description: String },
    #[error("Url error: {description}")]
    Url { description: String },
    #[error("Entry error: {description}")]
    Entry { description: String },
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

    pub fn blob_ticket(error: impl Display) -> Self {
        IrohError::BlobTicket {
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

    pub fn collection(error: impl Display) -> Self {
        IrohError::Collection {
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

    pub fn socket_addr(error: impl Display) -> Self {
        IrohError::SocketAddr {
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

    pub fn fs_util(error: impl Display) -> Self {
        IrohError::FsUtil {
            description: error.to_string(),
        }
    }

    pub fn tags(error: impl Display) -> Self {
        IrohError::Tags {
            description: error.to_string(),
        }
    }

    pub fn url(error: impl Display) -> Self {
        IrohError::Url {
            description: error.to_string(),
        }
    }

    pub fn entry(error: impl Display) -> Self {
        IrohError::Entry {
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
