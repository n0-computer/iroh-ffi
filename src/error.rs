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

impl From<uniffi::UnexpectedUniFFICallbackError> for IrohError {
    fn from(value: uniffi::UnexpectedUniFFICallbackError) -> Self {
        IrohError {
            e: anyhow::Error::from(value),
        }
    }
}
