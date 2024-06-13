use std::sync::Arc;

use crate::{block_on, BlobFormat, Hash, IrohError, IrohNode};
use bytes::Bytes;
use futures::TryStreamExt;

/// A response to a list collections request
pub struct TagInfo {
    /// The tag
    pub name: Vec<u8>,
    /// The format of the associated blob
    pub format: BlobFormat,
    /// The hash of the associated blob
    pub hash: Arc<Hash>,
}

impl From<iroh::client::tags::TagInfo> for TagInfo {
    fn from(res: iroh::client::tags::TagInfo) -> Self {
        TagInfo {
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
    pub fn tags_list(&self) -> Result<Vec<TagInfo>, IrohError> {
        block_on(&self.rt(), async {
            let tags = self
                .sync_client
                .tags()
                .list()
                .await?
                .map_ok(|l| l.into())
                .try_collect::<Vec<_>>()
                .await?;
            Ok(tags)
        })
    }

    /// Delete a tag
    pub fn tags_delete(&self, name: Vec<u8>) -> Result<(), IrohError> {
        let tag = iroh::blobs::Tag(Bytes::from(name));
        block_on(&self.rt(), async {
            self.sync_client.tags().delete(tag).await?;
            Ok(())
        })
    }
}
