use std::{str::FromStr, sync::Arc};

use futures::TryStreamExt;
use napi_derive::napi;

use crate::{block_on, IrohError, IrohNode};

/// Identifier for an [`Author`]
#[napi]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorId(pub(crate) iroh::sync::AuthorId);

impl std::fmt::Display for AuthorId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[napi]
impl IrohNode {
    /// Create a new author.
    pub fn author_create(&self) -> Result<Arc<AuthorId>, IrohError> {
        block_on(&self.rt(), async {
            let author = self
                .sync_client
                .authors
                .create()
                .await
                .map_err(IrohError::author)?;

            Ok(Arc::new(AuthorId(author)))
        })
    }

    /// List all the AuthorIds that exist on this node.
    pub fn author_list(&self) -> Result<Vec<Arc<AuthorId>>, IrohError> {
        block_on(&self.rt(), async {
            let authors = self
                .sync_client
                .authors
                .list()
                .await
                .map_err(IrohError::author)?
                .map_ok(|id| Arc::new(AuthorId(id)))
                .try_collect::<Vec<_>>()
                .await
                .map_err(IrohError::author)?;
            Ok(authors)
        })
    }
}

#[napi]
impl AuthorId {
    /// Get an [`AuthorId`] from a String.
    #[napi]
    pub fn from_string(str: String) -> Result<Self, IrohError> {
        let author = iroh::sync::AuthorId::from_str(&str).map_err(IrohError::author)?;
        Ok(AuthorId(author))
    }

    /// Returns true when both AuthorId's have the same value
    #[napi]
    pub fn equal(&self, other: &AuthorId) -> bool {
        *self == *other
    }
}
