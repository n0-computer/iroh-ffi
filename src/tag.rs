use std::sync::Arc;

use crate::{block_on, BlobFormat, Hash, IrohError, IrohNode};
use futures::TryStreamExt;

/// A response to a list collections request
pub struct ListTagsResponse {
    /// The tag
    pub name: Vec<u8>,
    /// The format of the associated blob
    pub format: BlobFormat,
    /// The hash of the associated blob
    pub hash: Arc<Hash>,
}

impl From<iroh::rpc_protocol::ListTagsResponse> for ListTagsResponse {
    fn from(res: iroh::rpc_protocol::ListTagsResponse) -> Self {
        ListTagsResponse {
            name: res.name.0.to_vec(),
            format: res.format.into(),
            hash: Arc::new(res.hash.into()),
        }
    }
}

impl IrohNode {
    /// List all tags
    ///
    /// Note: this allocates for each `ListTagsResponse`, if you have many `Tags`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    pub fn tags_list(&self) -> Result<Vec<ListTagsResponse>, IrohError> {
        block_on(&self.async_runtime, async {
            let tags = self
                .sync_client
                .tags
                .list()
                .await
                .map_err(IrohError::tags)?
                .map_ok(|l| l.into())
                .try_collect::<Vec<_>>()
                .await
                .map_err(IrohError::tags)?;
            Ok(tags)
        })
    }

    /// Delete a tag
    pub fn tags_delete(&self, name: Vec<u8>) -> Result<(), IrohError> {
        let tag = iroh::bytes::Tag(bytes::Bytes::from(name));
        block_on(&self.async_runtime, async {
            self.sync_client
                .tags
                .delete(tag)
                .await
                .map_err(IrohError::tags)
        })
    }
}
