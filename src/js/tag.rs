use crate::{BlobFormat, IrohNode};

use futures::TryStreamExt;

use napi::bindgen_prelude::Buffer;
use napi_derive::napi;

/// A response to a list collections request
#[napi(js_name = "ListTagsResponse")]
pub struct JsListTagsResponse {
    /// The tag
    pub name: Buffer,
    /// The format of the associated blob
    pub format: BlobFormat,
    /// The hash of the associated blob
    pub hash: String,
}

impl From<iroh::rpc_protocol::ListTagsResponse> for JsListTagsResponse {
    fn from(res: iroh::rpc_protocol::ListTagsResponse) -> Self {
        JsListTagsResponse {
            name: res.name.0.to_vec().into(),
            format: res.format.into(),
            hash: res.hash.to_string(),
        }
    }
}

#[napi]
impl IrohNode {
    #[napi(js_name = "tagsList")]
    pub async fn tags_list_js(&self) -> napi::Result<Vec<JsListTagsResponse>> {
        let tags = self
            .sync_client
            .tags
            .list()
            .await?
            .map_ok(|l| l.into())
            .try_collect::<Vec<_>>()
            .await?;
        Ok(tags)
    }

    #[napi(js_name = "tagsDelete")]
    pub async fn tags_delete_js(&self, name: Buffer) -> napi::Result<()> {
        let name: Vec<_> = name.into();
        let name = iroh::bytes::Tag(bytes::Bytes::from(name));
        self.sync_client.tags.delete(name).await?;
        Ok(())
    }
}
