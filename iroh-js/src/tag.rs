use bytes::Bytes;
use futures::TryStreamExt;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{BlobFormat, Iroh, TagsClient};

/// A response to a list collections request
#[derive(Debug)]
#[napi]
pub struct TagInfo {
    /// The tag
    pub name: Vec<u8>,
    /// The format of the associated blob
    pub format: BlobFormat,
    /// The hash of the associated blob
    pub hash: String,
}

impl From<iroh_blobs::rpc::client::tags::TagInfo> for TagInfo {
    fn from(res: iroh_blobs::rpc::client::tags::TagInfo) -> Self {
        TagInfo {
            name: res.name.0.to_vec(),
            format: res.format.into(),
            hash: res.hash.to_string(),
        }
    }
}

/// Iroh tags client.
#[napi]
pub struct Tags {
    client: TagsClient,
}

#[napi]
impl Iroh {
    /// Access to tags specific funtionaliy.
    pub fn tags(&self) -> Tags {
        Tags {
            client: self.tags_client.clone(),
        }
    }
}

#[napi]
impl Tags {
    /// List all tags
    ///
    /// Note: this allocates for each `ListTagsResponse`, if you have many `Tags`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    #[napi]
    pub async fn list(&self) -> Result<Vec<TagInfo>> {
        let tags = self
            .client
            .list()
            .await?
            .map_ok(|l| l.into())
            .try_collect::<Vec<_>>()
            .await?;
        Ok(tags)
    }

    /// Delete a tag
    #[napi]
    pub async fn delete(&self, name: Vec<u8>) -> Result<()> {
        let tag = iroh_blobs::Tag(Bytes::from(name));
        self.client.delete(tag).await?;
        Ok(())
    }
}
