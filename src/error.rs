/// An Error.
#[derive(Debug, thiserror::Error)]
#[error("{e:?}")]
pub struct IrohError {
    e: anyhow::Error,
}

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

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
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
