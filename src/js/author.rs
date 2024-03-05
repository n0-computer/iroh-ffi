use futures::TryStreamExt;
use napi_derive::napi;

use crate::{AuthorId, IrohNode};

#[napi]
impl IrohNode {
    /// Create a new author.
    #[napi(js_name = "authorCreate")]
    pub async fn author_create_js(&self) -> napi::Result<AuthorId> {
        let author = self.sync_client.authors.create().await?;
        Ok(AuthorId(author))
    }

    /// List all the AuthorIds that exist on this node.
    #[napi(js_name = "authorList")]
    pub async fn author_list_js(&self) -> napi::Result<Vec<AuthorId>> {
        let authors = self
            .sync_client
            .authors
            .list()
            .await?
            .map_ok(AuthorId)
            .try_collect::<Vec<_>>()
            .await?;

        Ok(authors)
    }
}

#[napi]
impl AuthorId {
    /// String representation
    #[napi(js_name = "toString")]
    pub fn to_string_js(&self) -> String {
        self.to_string()
    }
}
