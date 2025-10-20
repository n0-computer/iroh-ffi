use iroh_blobs::api::{ExportBaoError, RequestError};

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

impl From<RequestError> for IrohError {
    fn from(value: RequestError) -> Self {
        Self {
            e: anyhow::anyhow!("{:?}", value),
        }
    }
}

impl From<irpc::Error> for IrohError {
    fn from(value: irpc::Error) -> Self {
        Self {
            e: anyhow::anyhow!("{:?}", value),
        }
    }
}

impl From<ExportBaoError> for IrohError {
    fn from(value: ExportBaoError) -> Self {
        Self {
            e: anyhow::anyhow!("{:?}", value),
        }
    }
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
