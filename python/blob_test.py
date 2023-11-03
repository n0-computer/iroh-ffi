# tests that correspond to the `src/doc.rs` rust api
from iroh import Hash, IrohNode, SetTagOption, BlobFormat
import pytest
import tempfile
import random

def test_hash():
    hash_str = "bafkr4ih6qxpyfyrgxbcrvmiqbm7hb5fdpn4yezj7ayh6gwto4hm2573glu"
    hex_str = "fe85df82e226b8451ab1100b3e70f4a37b7982653f060fe35a6ee1d9aeff665d"
    bytes = b'\xfe\x85\xdf\x82\xe2\x26\xb8\x45\x1a\xb1\x10\x0b\x3e\x70\xf4\xa3\x7b\x79\x82\x65\x3f\x06\x0f\xe3\x5a\x6e\xe1\xd9\xae\xff\x66\x5d'
    cid_prefix = b'\x01\x55\x1e\x20'
    #
    # create hash from string
    hash = Hash.from_string(hash_str)
    #
    # test methods are as expected
    assert hash.to_string() == hash_str
    assert hash.to_bytes() == bytes
    assert hash.to_hex() == hex_str 
    assert hash.as_cid_bytes() == cid_prefix + bytes
    #
    # create hash from bytes
    hash_0 = Hash.from_bytes(bytes)
    #
    # test methods are as expected
    assert hash_0.to_string() == hash_str
    assert hash_0.to_bytes() == bytes
    assert hash_0.to_hex() == hex_str 
    assert hash_0.as_cid_bytes() == cid_prefix + bytes
    #
    # create hash from cid bytes
    hash_1 = Hash.from_cid_bytes(cid_prefix + bytes)
    #
    # test methods are as expected
    assert hash_1.to_string() == hash_str
    assert hash_1.to_bytes() == bytes
    assert hash_1.to_hex() == hex_str 
    assert hash_1.as_cid_bytes() == cid_prefix + bytes
    #
    # test that the eq function works
    assert hash.equal(hash_0)
    assert hash.equal(hash_1)
    assert hash_0.equal(hash)
    assert hash_0.equal(hash_1)
    assert hash_1.equal(hash)
    assert hash_1.equal(hash_0)
 
# test functionality between adding as bytes and reading to bytes
def test_blob_add_get_bytes():
    #
    # create node
    dir = tempfile.mkdtemp()
    node = IrohNode(dir)
    tag = SetTagOption.auto()
    #
    # create bytes
    blob_size = 100
    bytes = bytearray(map(random.getrandbits,(8,)*blob_size))
    #
    # add blob
    tag = SetTagOption.auto()
    add_outcome = node.blobs_add_bytes(bytes, tag)
    #
    # check outcome info is as expected
    assert add_outcome.format == BlobFormat.RAW
    # assert add_outcome.size == blob_size
    #
    # check we get the expected size from the hash
    hash = add_outcome.hash
    got_size = node.blobs_size(hash)
    # assert got_size == blob_size
    #
    # get bytes
    got_bytes = node.blobs_read_to_bytes(hash)
    assert len(got_bytes) == blob_size
    assert got_bytes == bytes

# test functionality between reading bytes from a path and writing bytes to
# a path
