use std::sync::Arc;

use crate::{BlobFormat, Hash, Iroh, IrohError, Storage};
use bytes::Bytes;
use futures::TryStreamExt;
use quic_rpc::transport::flume::FlumeConnector;

type MemClient = iroh_blobs::rpc::client::tags::Client<
    FlumeConnector<iroh_blobs::rpc::proto::Response, iroh_blobs::rpc::proto::Request>,
>;

/// A response to a list collections request
#[derive(Debug, uniffi::Record)]
pub struct TagInfo {
    /// The tag
    pub name: Vec<u8>,
    /// The format of the associated blob
    pub format: BlobFormat,
    /// The hash of the associated blob
    pub hash: Arc<Hash>,
}

impl From<iroh_blobs::rpc::client::tags::TagInfo> for TagInfo {
    fn from(res: iroh_blobs::rpc::client::tags::TagInfo) -> Self {
        TagInfo {
            name: res.name.0.to_vec(),
            format: res.format.into(),
            hash: Arc::new(res.hash.into()),
        }
    }
}

/// Iroh tags client.
#[derive(uniffi::Object)]
pub struct Tags {
    tags: MemClient,
}

#[uniffi::export]
impl Iroh {
    /// Access to tags specific funtionaliy.
    pub fn tags(&self) -> Tags {
        let client = match self.storage {
            Storage::Fs => {
                let blobs = self
                    .get_protocol::<iroh_blobs::net_protocol::Blobs<iroh_blobs::store::fs::Store>>(
                        iroh_blobs::protocol::ALPN,
                    )
                    .expect("missing blobs");
                blobs.client()
            }
            Storage::Memory => {
                let blobs = self
                    .get_protocol::<iroh_blobs::net_protocol::Blobs<iroh_blobs::store::mem::Store>>(
                        iroh_blobs::protocol::ALPN,
                    )
                    .expect("missing blobs");
                blobs.client()
            }
        };
        let tags = client.tags();

        Tags { tags }
    }
}

impl Tags {
    fn client(&self) -> &MemClient {
        &self.tags
    }
}

#[uniffi::export]
impl Tags {
    /// List all tags
    ///
    /// Note: this allocates for each `ListTagsResponse`, if you have many `Tags`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn list(&self) -> Result<Vec<TagInfo>, IrohError> {
        let tags = self
            .client()
            .list()
            .await?
            .map_ok(|l| l.into())
            .try_collect::<Vec<_>>()
            .await?;
        Ok(tags)
    }

    /// Delete a tag
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn delete(&self, name: Vec<u8>) -> Result<(), IrohError> {
        let tag = iroh_blobs::Tag(Bytes::from(name));
        self.client().delete(tag).await?;
        Ok(())
    }
}
