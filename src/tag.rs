use std::sync::Arc;

use crate::{block_on, BlobFormat, Hash, IrohError, IrohNode};
use futures::TryStreamExt;

/// A response to a list collections request
pub struct ListTagsResponse {
    /// The tag
    pub name: Arc<Tag>,
    /// The format of the associated blob
    pub format: BlobFormat,
    /// The hash of the associated blob
    pub hash: Arc<Hash>,
}

impl From<iroh::rpc_protocol::ListTagsResponse> for ListTagsResponse {
    fn from(res: iroh::rpc_protocol::ListTagsResponse) -> Self {
        ListTagsResponse {
            name: Arc::new(res.name.into()),
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
    pub fn tags_delete(&self, name: Arc<Tag>) -> Result<(), IrohError> {
        block_on(&self.async_runtime, async {
            self.sync_client
                .tags
                .delete(name.0.clone())
                .await
                .map_err(IrohError::tags)
        })
    }
}

/// A tag
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag(pub(crate) iroh::bytes::Tag);

impl Tag {
    /// Create a tag from a String
    pub fn from_string(t: String) -> Self {
        let tag: iroh::bytes::Tag = t.into();
        tag.into()
    }

    /// Create a tag from a byte array
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let tag: iroh::bytes::Tag = bytes::Bytes::from(bytes).into();
        tag.into()
    }

    /// Serialize a tag as a byte array
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0 .0.to_vec()
    }

    /// Returns true if the Tags have the same value
    pub fn equal(&self, other: Arc<Tag>) -> bool {
        *self == *other
    }
}

impl From<iroh::bytes::Tag> for Tag {
    fn from(t: iroh::bytes::Tag) -> Self {
        Tag(t)
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Tag> for iroh::bytes::Tag {
    fn from(value: Tag) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag() {
        let tag_str = "\"foo\"".to_string();
        let bytes = b"foo".to_vec();

        // create tag from string
        let tag = Tag::from_string("foo".to_string());

        // test methods are as expected
        assert_eq!(tag_str, tag.to_string());
        assert_eq!(bytes, tag.to_bytes());

        // create tag from bytes
        let tag_0 = Arc::new(Tag::from_bytes(bytes.clone()));

        // test methods are as expected
        assert_eq!(tag_str, tag_0.to_string());
        assert_eq!(bytes, tag_0.to_bytes());

        // test that the eq function works
        assert!(tag.equal(tag_0.clone()));
        assert!(tag_0.equal(tag.into()));
    }
}
