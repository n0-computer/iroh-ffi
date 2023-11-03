# tests that correspond to the `src/doc.rs` rust api
import pytest
import tempfile
import random
import os

from iroh import Hash, IrohNode, SetTagOption, BlobFormat, WrapOption, AddProgressType

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
    dir = tempfile.TemporaryDirectory()
    node = IrohNode(dir.name)
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
def test_blob_read_write_path():
    iroh_dir = tempfile.TemporaryDirectory()
    node = IrohNode(iroh_dir.name)
    #
    # create bytes
    blob_size = 100
    bytes = bytearray(map(random.getrandbits,(8,)*blob_size))
    # 
    # write to file
    dir = tempfile.TemporaryDirectory()
    path = os.path.join(dir.name, "in")
    file = open(path, "wb")
    file.write(bytes)
    file.close()
    #
    # add blob
    tag = SetTagOption.auto()
    wrap = WrapOption.no_wrap()

    class AddCallback:
        hash = None
        format = None

        def progress(x, progress_event):
            print(progress_event.type())
            if progress_event.type() == AddProgressType.ALL_DONE:
                all_done_event = progress_event.as_all_done()
                x.hash = all_done_event.hash
                print(all_done_event.hash)
                print(all_done_event.format)
                x.format = all_done_event.format
            if progress_event.type() == AddProgressType.ABORT:
                abort_event = progress_event.as_abort()
                raise Exception(abort_event.error)

    cb = AddCallback()
    node.blobs_add_from_path(path, False, tag, wrap, cb)
    #
    # check outcome info is as expected
    assert cb.format == BlobFormat.RAW
    assert cb.hash != None
    #
    # check we get the expected size from the hash
    got_size = node.blobs_size(cb.hash)
    assert got_size == blob_size
    #
    # get bytes
    got_bytes = node.blobs_read_to_bytes(cb.hash)
    print("read_to_bytes {}", got_bytes)
    assert len(got_bytes) == blob_size
    assert got_bytes == bytes
    #
    # write to file
    out_path = os.path.join(dir.name, "out")
    node.blobs_write_to_path(cb.hash, out_path)
    # open file
    got_file = open(out_path, "rb")
    got_bytes = got_file.read()
    got_file.close()
    print("write_to_path {}", got_bytes)
    assert len(got_bytes) == blob_size
    assert got_bytes == bytes

# def test_blob_collections():
        # make folder structure
    # add from path
    # get hash
    # list collections
    # ensure it's in there

def test_list_and_delete():
    iroh_dir = tempfile.TemporaryDirectory()
    node = IrohNode(iroh_dir.name)
    #
    # create bytes
    blob_size = 100
    blobs = []
    num_blobs = 3;

    for x in range(num_blobs):
        print(x)
        bytes = bytearray(map(random.getrandbits,(8,)*blob_size))
        blobs.append(bytes)

    hashes = []
    for blob in blobs:
        output = node.blobs_add_bytes(blob, SetTagOption.auto())
        hashes.append(output.hash)

    list = node.blobs_list()
    assert len(list) == num_blobs 
    hashes_exist(hashes, list)

    remove_hash = hashes.pop(0)
    node.blobs_delete_blob(remove_hash)

    list = node.blobs_list();
    assert len(list) == num_blobs - 1
    hashes_exist(hashes, list)

    for hash in list:
        if remove_hash.equal(hash):
            raise Exception("blob {} should have been removed", remove_hash)

def hashes_exist(expect, got):
    for hash in expect:
        exists = False
        for h in got:
            if h.equal(hash):
                exists = True
        if not exists:
            raise Exception("could not find {} in list", hash)



# def test_download():
    # can't test this until can adjust ports

