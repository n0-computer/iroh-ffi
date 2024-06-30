use std::{str::FromStr, sync::Arc};

use futures::TryStreamExt;

use crate::{block_on, IrohError, IrohNode};

/// Identifier for an [`Author`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorId(pub(crate) iroh::docs::AuthorId);

impl std::fmt::Display for AuthorId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AuthorId {
    /// Get an [`AuthorId`] from a String.
    pub fn from_string(str: String) -> Result<Self, IrohError> {
        let author = iroh::docs::AuthorId::from_str(&str)?;
        Ok(AuthorId(author))
    }

    /// Returns true when both AuthorId's have the same value
    pub fn equal(&self, other: &AuthorId) -> bool {
        *self == *other
    }
}

/// Author key to insert entries in a document
///
/// Internally, an author is a `SigningKey` which is used to sign entries.
#[derive(Debug, Clone)]
pub struct Author(pub(crate) iroh::docs::Author);

impl Author {
    /// Get an [`Author`] from a String
    pub fn from_string(str: String) -> Result<Self, IrohError> {
        let author = iroh::docs::Author::from_str(&str)?;
        Ok(Author(author))
    }

    /// Get the [`AuthorId`] of this Author
    pub fn id(&self) -> Arc<AuthorId> {
        Arc::new(AuthorId(self.0.id()))
    }
}

impl std::fmt::Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[uniffi::export]
impl IrohNode {
    /// Returns the default document author of this node.
    ///
    /// On persistent nodes, the author is created on first start and its public key is saved
    /// in the data directory.
    ///
    /// The default author can be set with [`Self::set_default`].
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn author_default(&self) -> Result<Arc<AuthorId>, IrohError> {
        let author = self.sync_client.authors().default().await?;
        Ok(Arc::new(AuthorId(author)))
    }

    /// List all the AuthorIds that exist on this node.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn author_list(&self) -> Result<Vec<Arc<AuthorId>>, IrohError> {
        let authors = self
            .sync_client
            .authors()
            .list()
            .await?
            .map_ok(|id| Arc::new(AuthorId(id)))
            .try_collect::<Vec<_>>()
            .await?;
        Ok(authors)
    }
}

impl IrohNode {
    /// Create a new document author.
    ///
    /// You likely want to save the returned [`AuthorId`] somewhere so that you can use this author
    /// again.
    ///
    /// If you need only a single author, use [`Self::default`].
    pub fn author_create(&self) -> Result<Arc<AuthorId>, IrohError> {
        block_on(&self.rt(), async {
            let author = self.sync_client.authors().create().await?;

            Ok(Arc::new(AuthorId(author)))
        })
    }

    /// Export the given author.
    ///
    /// Warning: This contains sensitive data.
    pub fn author_export(&self, author: Arc<AuthorId>) -> Result<Arc<Author>, IrohError> {
        block_on(&self.rt(), async {
            let author = self.sync_client.authors().export(author.0).await?;
            match author {
                Some(author) => Ok(Arc::new(Author(author))),
                None => Err(anyhow::anyhow!("Author Not Found").into()),
            }
        })
    }

    /// Import the given author.
    ///
    /// Warning: This contains sensitive data.
    pub fn author_import(&self, author: Arc<Author>) -> Result<Arc<AuthorId>, IrohError> {
        block_on(&self.rt(), async {
            self.sync_client.authors().import(author.0.clone()).await?;
            Ok(Arc::new(AuthorId(author.0.id())))
        })
    }

    /// Deletes the given author by id.
    ///
    /// Warning: This permanently removes this author.
    pub fn author_delete(&self, author: Arc<AuthorId>) -> Result<(), IrohError> {
        block_on(&self.rt(), async {
            self.sync_client.authors().delete(author.0).await?;
            Ok(())
        })
    }
}

mod tests {
    #[test]
    fn test_author_api() {
        let dir = tempfile::tempdir().unwrap();
        let node = crate::IrohNode::new(dir.into_path().display().to_string()).unwrap();

        assert_eq!(node.author_list().unwrap().len(), 1);
        let author_id = node.author_create().unwrap();
        let authors = node.author_list().unwrap();
        assert_eq!(authors.len(), 2);
        let author = node.author_export(author_id.clone()).unwrap();
        assert!(author_id.equal(&author.id()));
        node.author_delete(author_id).unwrap();
        let authors = node.author_list().unwrap();
        assert_eq!(authors.len(), 1);
        node.author_import(author).unwrap();
        let authors = node.author_list().unwrap();
        assert_eq!(authors.len(), 2);
    }
}
