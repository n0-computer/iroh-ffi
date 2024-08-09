use futures::TryStreamExt;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::Iroh;

/// Identifier for an [`Author`]
#[derive(Debug, Clone, PartialEq, Eq)]
#[napi]
pub struct AuthorId(pub(crate) iroh::docs::AuthorId);

#[napi]
impl AuthorId {
    /// Get an [`AuthorId`] from a String.
    #[napi(factory)]
    pub fn from_string(str: String) -> Result<Self> {
        let author: iroh::docs::AuthorId = str.parse()?;
        Ok(AuthorId(author))
    }

    /// Returns true when both AuthorId's have the same value
    #[napi]
    pub fn is_equal(&self, other: &AuthorId) -> bool {
        *self == *other
    }

    #[napi]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

/// Author key to insert entries in a document
///
/// Internally, an author is a `SigningKey` which is used to sign entries.
#[derive(Debug, Clone)]
#[napi]
pub struct Author(pub(crate) iroh::docs::Author);

#[napi]
impl Author {
    /// Get an [`Author`] from a String
    #[napi(factory)]
    pub fn from_string(str: String) -> Result<Self> {
        let author: iroh::docs::Author = str.parse()?;
        Ok(Author(author))
    }

    /// Get the [`AuthorId`] of this Author
    #[napi]
    pub fn id(&self) -> AuthorId {
        AuthorId(self.0.id())
    }

    #[napi]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

/// Iroh authors client.
#[napi]
pub struct Authors {
    node: Iroh,
}

#[napi]
impl Iroh {
    /// Access to authors specific funtionaliy.
    #[napi(getter)]
    pub fn authors(&self) -> Authors {
        Authors { node: self.clone() }
    }
}

impl Authors {
    fn client(&self) -> &iroh::client::Iroh {
        self.node.client()
    }
}

#[napi]
impl Authors {
    /// Returns the default document author of this node.
    ///
    /// On persistent nodes, the author is created on first start and its public key is saved
    /// in the data directory.
    ///
    /// The default author can be set with [`Self::set_default`].
    #[napi]
    pub async fn default(&self) -> Result<AuthorId> {
        let author = self.client().authors().default().await?;
        Ok(AuthorId(author))
    }

    /// List all the AuthorIds that exist on this node.
    #[napi]
    pub async fn list(&self) -> Result<Vec<AuthorId>> {
        let authors = self
            .client()
            .authors()
            .list()
            .await?
            .map_ok(|id| AuthorId(id))
            .try_collect::<Vec<_>>()
            .await?;
        Ok(authors)
    }

    /// Create a new document author.
    ///
    /// You likely want to save the returned [`AuthorId`] somewhere so that you can use this author
    /// again.
    ///
    /// If you need only a single author, use [`Self::default`].
    #[napi]
    pub async fn create(&self) -> Result<AuthorId> {
        let author = self.client().authors().create().await?;

        Ok(AuthorId(author))
    }

    /// Export the given author.
    ///
    /// Warning: This contains sensitive data.
    #[napi]
    pub async fn export(&self, author: &AuthorId) -> Result<Author> {
        let author = self.client().authors().export(author.0).await?;
        match author {
            Some(author) => Ok(Author(author)),
            None => Err(anyhow::anyhow!("Author Not Found").into()),
        }
    }

    /// Import the given author.
    ///
    /// Warning: This contains sensitive data.
    #[napi]
    pub async fn import(&self, author: &Author) -> Result<AuthorId> {
        self.client().authors().import(author.0.clone()).await?;
        Ok(AuthorId(author.0.id()))
    }

    /// Deletes the given author by id.
    ///
    /// Warning: This permanently removes this author.
    #[napi]
    pub async fn delete(&self, author: &AuthorId) -> Result<()> {
        self.client().authors().delete(author.0).await?;
        Ok(())
    }
}
