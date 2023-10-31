# tests that correspond to the `src/doc.rs` rust api
from iroh import Tag 
import pytest

def test_tag():
    tag_str = "\"foo\""
    bytes = b'foo'
    #
    # create tag from string
    tag = Tag.from_string("foo")
    #
    # test methods are as expected
    assert tag.to_string() == tag_str
    assert tag.to_bytes() == bytes
    #
    # create tag from bytes
    tag_0 = Tag.from_bytes(bytes)
    #
    # test methods are as expected
    assert tag_0.to_string() == tag_str
    assert tag_0.to_bytes() == bytes
    #
    # test that the eq function works
    assert tag.equal(tag_0)
    assert tag_0.equal(tag)

