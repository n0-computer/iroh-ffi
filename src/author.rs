use std::{str::FromStr, sync::Arc};

use futures::TryStreamExt;

use crate::{IrohError, IrohNode};

/// Identifier for an [`Author`]
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Object)]
pub struct AuthorId(pub(crate) iroh::docs::AuthorId);

impl std::fmt::Display for AuthorId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[uniffi::export]
impl AuthorId {
    /// Get an [`AuthorId`] from a String.
    #[uniffi::constructor]
    pub fn from_string(str: String) -> Result<Self, IrohError> {
        let author = iroh::docs::AuthorId::from_str(&str)?;
        Ok(AuthorId(author))
    }

    /// Returns true when both AuthorId's have the same value
    #[uniffi::method]
    pub fn equal(&self, other: &AuthorId) -> bool {
        *self == *other
    }
}

/// Author key to insert entries in a document
///
/// Internally, an author is a `SigningKey` which is used to sign entries.
#[derive(Debug, Clone, uniffi::Object)]
pub struct Author(pub(crate) iroh::docs::Author);

#[uniffi::export]
impl Author {
    /// Get an [`Author`] from a String
    #[uniffi::constructor]
    pub fn from_string(str: String) -> Result<Self, IrohError> {
        let author = iroh::docs::Author::from_str(&str)?;
        Ok(Author(author))
    }

    /// Get the [`AuthorId`] of this Author
    #[uniffi::method]
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
        let author = self.node.authors().default().await?;
        Ok(Arc::new(AuthorId(author)))
    }

    /// List all the AuthorIds that exist on this node.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn author_list(&self) -> Result<Vec<Arc<AuthorId>>, IrohError> {
        let authors = self
            .node
            .authors()
            .list()
            .await?
            .map_ok(|id| Arc::new(AuthorId(id)))
            .try_collect::<Vec<_>>()
            .await?;
        Ok(authors)
    }
}

#[uniffi::export]
impl IrohNode {
    /// Create a new document author.
    ///
    /// You likely want to save the returned [`AuthorId`] somewhere so that you can use this author
    /// again.
    ///
    /// If you need only a single author, use [`Self::default`].
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn author_create(&self) -> Result<Arc<AuthorId>, IrohError> {
        let author = self.node.authors().create().await?;

        Ok(Arc::new(AuthorId(author)))
    }

    /// Export the given author.
    ///
    /// Warning: This contains sensitive data.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn author_export(&self, author: Arc<AuthorId>) -> Result<Arc<Author>, IrohError> {
        let author = self.node.authors().export(author.0).await?;
        match author {
            Some(author) => Ok(Arc::new(Author(author))),
            None => Err(anyhow::anyhow!("Author Not Found").into()),
        }
    }

    /// Import the given author.
    ///
    /// Warning: This contains sensitive data.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn author_import(&self, author: Arc<Author>) -> Result<Arc<AuthorId>, IrohError> {
        self.node.authors().import(author.0.clone()).await?;
        Ok(Arc::new(AuthorId(author.0.id())))
    }

    /// Deletes the given author by id.
    ///
    /// Warning: This permanently removes this author.
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn author_delete(&self, author: Arc<AuthorId>) -> Result<(), IrohError> {
        self.node.authors().delete(author.0).await?;
        Ok(())
    }
}

mod tests {
    #[tokio::test]
    async fn test_author_api() {
        let dir = tempfile::tempdir().unwrap();
        let node = crate::IrohNode::new(dir.into_path().display().to_string())
            .await
            .unwrap();

        assert_eq!(node.author_list().await.unwrap().len(), 1);
        let author_id = node.author_create().await.unwrap();
        let authors = node.author_list().await.unwrap();
        assert_eq!(authors.len(), 2);
        let author = node.author_export(author_id.clone()).await.unwrap();
        assert!(author_id.equal(&author.id()));
        node.author_delete(author_id).await.unwrap();
        let authors = node.author_list().await.unwrap();
        assert_eq!(authors.len(), 1);
        node.author_import(author).await.unwrap();
        let authors = node.author_list().await.unwrap();
        assert_eq!(authors.len(), 2);
    }
}
