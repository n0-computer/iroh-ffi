use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use futures::{StreamExt, TryStreamExt};
use napi::bindgen_prelude::{Buffer, Generator};
use napi_derive::napi;

use crate::{
    BlobAddOutcome, BlobFormat, BlobListCollectionsResponse, BlobListIncompleteResponse,
    Collection, Hash, HashAndTag, IrohNode, NodeAddr,
};

#[napi]
impl IrohNode {
    /// List all complete blobs.
    ///
    /// Note: this allocates for each `BlobListResponse`, if you have many `BlobListReponse`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    #[napi(js_name = "blobsList")]
    pub async fn blobs_list_js(&self) -> napi::Result<Vec<String>> {
        let response = self.sync_client.blobs.list().await?;
        let hashes: Vec<String> = response
            .map_ok(|i| i.hash.to_string())
            .try_collect()
            .await?;

        Ok(hashes)
    }

    /// Get the size information on a single blob.
    ///
    /// Method only exists in FFI
    #[napi(js_name = "blobsSize")]
    pub async fn blobs_size_js(&self, hash: &Hash) -> napi::Result<u32> {
        let r = self.sync_client.blobs.read(hash.0).await?;
        let size = u32::try_from(r.size()).context("cannot convert blob size to u32")?;
        Ok(size)
    }

    /// Read all bytes of single blob.
    ///
    /// This allocates a buffer for the full blob. Use only if you know that the blob you're
    /// reading is small.
    #[napi(js_name = "blobsReadToBytes")]
    pub async fn blobs_read_to_bytes_js(&self, hash: &Hash) -> napi::Result<Buffer> {
        let res = self
            .sync_client
            .blobs
            .read_to_bytes(hash.0)
            .await
            .map(|b| b.to_vec())?;
        Ok(res.into())
    }

    /// Read all bytes of single blob at `offset` for length `len`.
    ///
    /// This allocates a buffer for the full length `len`. Use only if you know that the blob you're
    /// reading is small.
    #[napi(js_name = "blobsReadAtToBytes")]
    pub async fn blobs_read_at_to_bytes_js(
        &self,
        hash: &Hash,
        offset: u32,
        len: Option<u32>,
    ) -> Result<Buffer, napi::Error> {
        let len = len.map(|l| l as _);
        let res = self
            .sync_client
            .blobs
            .read_at_to_bytes(hash.0, offset as _, len)
            .await
            .map(|b| b.to_vec())?;
        Ok(res.into())
    }

    /// Import a blob from a filesystem path.
    ///
    /// `path` should be an absolute path valid for the file system on which
    /// the node runs.
    /// If `in_place` is true, Iroh will assume that the data will not change and will share it in
    /// place without copying to the Iroh data directory.
    #[napi(js_name = "blobsAddFromPath")]
    pub async fn blobs_add_from_path_js(
        &self,
        path: String,
        in_place: bool,
        tag: Option<Buffer>,
        wrap: bool,
    ) -> Result<JsAddProgress, napi::Error> {
        let tag = match tag {
            None => iroh::rpc_protocol::SetTagOption::Auto,
            Some(name) => {
                let name: Vec<_> = name.into();
                iroh::rpc_protocol::SetTagOption::Named(bytes::Bytes::from(name).into())
            }
        };
        let wrap = if wrap {
            iroh::rpc_protocol::WrapOption::Wrap { name: None }
        } else {
            iroh::rpc_protocol::WrapOption::NoWrap
        };
        let mut stream = self
            .sync_client
            .blobs
            .add_from_path(path.into(), in_place, tag, wrap)
            .await?;

        // arbitrary channel size
        let (send, recv) = flume::bounded(64);
        let handle = tokio::spawn(async move {
            while let Some(res) = stream.next().await {
                send.send(res).expect("receiver dropped");
            }
        });
        Ok(JsAddProgress { recv, handle })
    }

    /// Export the blob contents to a file path
    /// The `path` field is expected to be the absolute path.
    #[napi(js_name = "blobsWriteToPath")]
    pub async fn blobs_write_to_path_js(
        &self,
        hash: &Hash,
        path: String,
    ) -> Result<(), napi::Error> {
        let mut reader = self.sync_client.blobs.read(hash.0).await?;

        let path: PathBuf = path.into();
        if let Some(dir) = path.parent() {
            tokio::fs::create_dir_all(dir).await?;
        }
        let mut file = tokio::fs::File::create(path).await?;
        tokio::io::copy(&mut reader, &mut file).await?;

        Ok(())
    }

    /// Write a blob by passing bytes.
    #[napi(js_name = "blobsAddBytes")]
    pub async fn blobs_add_bytes_js(
        &self,
        bytes: Buffer,
        tag: Option<Buffer>,
    ) -> Result<serde_json::Value, napi::Error> {
        let tag = match tag {
            None => iroh::rpc_protocol::SetTagOption::Auto,
            Some(name) => {
                let name: Vec<_> = name.into();
                iroh::rpc_protocol::SetTagOption::Named(bytes::Bytes::from(name).into())
            }
        };
        let bytes: Vec<_> = bytes.into();
        let res = self
            .sync_client
            .blobs
            .add_bytes(bytes.into(), tag)
            .await
            .map(|outcome| serde_json::to_value(BlobAddOutcome::from(outcome)))??;
        Ok(res)
    }

    /// Download a blob from another node and add it to the local database.
    #[napi(js_name = "blobsDownload")]
    pub async fn blobs_download_js(
        &self,
        hash: &Hash,
        format: BlobFormat,
        node: &NodeAddr,
        tag: Option<Vec<u8>>,
        out: Option<String>,
        in_place: bool,
    ) -> Result<JsDownloadProgress, napi::Error> {
        let tag = match tag {
            None => iroh::rpc_protocol::SetTagOption::Auto,
            Some(name) => iroh::rpc_protocol::SetTagOption::Named(bytes::Bytes::from(name).into()),
        };
        let out = if let Some(out) = out {
            iroh::rpc_protocol::DownloadLocation::External {
                path: PathBuf::from(out),
                in_place,
            }
        } else {
            iroh::rpc_protocol::DownloadLocation::Internal
        };
        let req = iroh::rpc_protocol::BlobDownloadRequest {
            hash: hash.0,
            format: format.into(),
            peer: node.clone().try_into().unwrap(),
            tag,
            out,
        };
        let mut stream = self.sync_client.blobs.download(req).await?;
        // arbitrary channel size
        let (send, recv) = flume::bounded(64);
        let handle = tokio::spawn(async move {
            while let Some(res) = stream.next().await {
                send.send(res).expect("receiver dropped");
            }
        });
        Ok(JsDownloadProgress { recv, handle })
    }

    /// List all incomplete (partial) blobs.
    ///
    /// Note: this allocates for each `BlobListIncompleteResponse`, if you have many `BlobListIncompleteResponse`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    #[napi(js_name = "blobsListIncomplete")]
    pub async fn blobs_list_incomplete_js(&self) -> Result<Vec<serde_json::Value>, napi::Error> {
        let blobs = self
            .sync_client
            .blobs
            .list_incomplete()
            .await?
            .map_ok(|res| serde_json::to_value(BlobListIncompleteResponse::from(res)).unwrap())
            .try_collect::<Vec<_>>()
            .await?;
        Ok(blobs)
    }

    /// List all collections.
    ///
    /// Note: this allocates for each `BlobListCollectionsResponse`, if you have many `BlobListCollectionsResponse`s this may be a prohibitively large list.
    /// Please file an [issue](https://github.com/n0-computer/iroh-ffi/issues/new) if you run into this issue
    #[napi(js_name = "blobsListCollections")]
    pub async fn blobs_list_collections_js(&self) -> Result<Vec<serde_json::Value>, napi::Error> {
        let blobs = self
            .sync_client
            .blobs
            .list_collections()
            .await?
            .map_ok(|res| serde_json::to_value(BlobListCollectionsResponse::from(res)).unwrap())
            .try_collect::<Vec<_>>()
            .await?;
        Ok(blobs)
    }

    /// Read the content of a collection
    #[napi(js_name = "blobsGetCollection")]
    pub async fn blobs_get_collection_js(&self, hash: &Hash) -> Result<Collection, napi::Error> {
        let collection = self.sync_client.blobs.get_collection(hash.0).await?;

        Ok(collection.into())
    }

    /// Create a collection from already existing blobs.
    ///
    /// To automatically clear the tags for the passed in blobs you can set
    /// `tags_to_delete` on those tags, and they will be deleted once the collection is created.
    #[napi(js_name = "blobsCreateCollection")]
    pub async fn blobs_create_collection_js(
        &self,
        collection: &Collection,
        tag: Option<Vec<u8>>,
        tags_to_delete: Vec<String>,
    ) -> Result<serde_json::Value, napi::Error> {
        let collection = collection.0.read().unwrap().clone();
        let tag = match tag {
            None => iroh::rpc_protocol::SetTagOption::Auto,
            Some(name) => iroh::rpc_protocol::SetTagOption::Named(bytes::Bytes::from(name).into()),
        };

        let (hash, tag) = self
            .sync_client
            .blobs
            .create_collection(
                collection,
                tag,
                tags_to_delete
                    .into_iter()
                    .map(iroh::bytes::Tag::from)
                    .collect(),
            )
            .await?;

        Ok(serde_json::to_value(HashAndTag {
            hash: Arc::new(hash.into()),
            tag: tag.0.to_vec(),
        })
        .unwrap())
    }

    /// Delete a blob.
    #[napi(js_name = "blobsDeleteBlob")]
    pub async fn blobs_delete_blob_js(&self, hash: &Hash) -> Result<(), napi::Error> {
        self.sync_client
            .blobs
            .delete_blob((*hash).clone().0)
            .await?;
        Ok(())
    }
}

#[napi]
impl Hash {
    /// Bytes of the hash.
    #[napi(js_name = "toBytes")]
    pub fn to_bytes_js(&self) -> napi::bindgen_prelude::Buffer {
        self.0.as_bytes().to_vec().into()
    }

    /// Convert the hash to a string
    #[napi(js_name = "toString")]
    pub fn to_string_js(&self) -> String {
        format!("{}", self.0)
    }
}

#[napi]
impl Collection {
    /// Get the blobs associated with this collection
    #[napi(js_name = "blobs")]
    pub fn blobs_js(&self) -> napi::Result<Vec<JsLinkAndName>> {
        Ok(self
            .0
            .read()
            .map_err(|_| anyhow::anyhow!("poisoned read lock"))?
            .iter()
            .map(|(name, hash)| JsLinkAndName {
                name: name.clone(),
                link: hash.to_hex(),
            })
            .collect())
    }

    /// Returns the number of blobs in this collection
    #[napi(js_name = "len")]
    pub fn len_js(&self) -> u32 {
        self.0.read().unwrap().len() as _
    }
}

#[napi(iterator)]
pub struct JsAddProgress {
    recv: flume::Receiver<anyhow::Result<iroh::rpc_protocol::AddProgress>>,
    handle: tokio::task::JoinHandle<()>,
}

#[napi]
impl Generator for JsAddProgress {
    type Yield = serde_json::Value;
    type Next = serde_json::Value;
    type Return = serde_json::Value;

    fn next(&mut self, _value: Option<Self::Next>) -> Option<Self::Yield> {
        self.recv
            .recv()
            .ok()
            .and_then(|event| event.ok())
            .and_then(|event| serde_json::to_value(event).ok())
    }

    fn complete(&mut self, _value: Option<Self::Return>) -> Option<Self::Yield> {
        let mut res = None;
        while let Ok(Ok(progress)) = self.recv.recv() {
            match progress {
                iroh::rpc_protocol::AddProgress::AllDone { .. }
                | iroh::rpc_protocol::AddProgress::Abort(_) => {
                    res = serde_json::to_value(progress).ok();
                    break;
                }
                _ => {}
            }
        }
        self.handle.abort();
        res
    }

    fn catch(
        &mut self,
        _env: napi::Env,
        value: napi::JsUnknown,
    ) -> Result<Option<Self::Yield>, napi::JsUnknown> {
        self.handle.abort();
        Err(value)
    }
}

#[napi(iterator)]
pub struct JsDownloadProgress {
    recv: flume::Receiver<anyhow::Result<iroh::rpc_protocol::DownloadProgress>>,
    handle: tokio::task::JoinHandle<()>,
}

#[napi]
impl Generator for JsDownloadProgress {
    type Yield = serde_json::Value;
    type Next = serde_json::Value;
    type Return = serde_json::Value;

    fn next(&mut self, _value: Option<Self::Next>) -> Option<Self::Yield> {
        self.recv
            .recv()
            .ok()
            .and_then(|event| event.ok())
            .and_then(|event| serde_json::to_value(event).ok())
    }

    fn complete(&mut self, _value: Option<Self::Return>) -> Option<Self::Yield> {
        let mut res = None;
        while let Ok(Ok(progress)) = self.recv.recv() {
            match progress {
                iroh::rpc_protocol::DownloadProgress::AllDone { .. }
                | iroh::rpc_protocol::DownloadProgress::Abort(_) => {
                    res = serde_json::to_value(progress).ok();
                    break;
                }
                _ => {}
            }
        }
        self.handle.abort();
        res
    }

    fn catch(
        &mut self,
        _env: napi::Env,
        value: napi::JsUnknown,
    ) -> Result<Option<Self::Yield>, napi::JsUnknown> {
        self.handle.abort();
        Err(value)
    }
}

/// `LinkAndName` includes a name and a hash for a blob in a collection
#[napi(js_name = "LinkAndName")]
#[derive(Clone, Debug)]
pub struct JsLinkAndName {
    /// The name associated with this [`Hash`].
    pub name: String,
    /// The [`Hash`] of the blob.
    pub link: String,
}
