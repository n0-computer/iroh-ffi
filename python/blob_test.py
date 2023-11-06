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
    assert got_size == blob_size
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

def test_blob_collections():
    collection_dir = tempfile.TemporaryDirectory()
    num_files = 3
    blob_size = 100
    for i in range(num_files):
        path = os.path.join(collection_dir.name, str(i))
        bytes = bytearray(map(random.getrandbits,(8,)*blob_size))
        file = open(path, "wb")
        file.write(bytes)
        file.close()
    print(collection_dir.__sizeof__())

    # make node
    iroh_dir = tempfile.TemporaryDirectory()
    node = IrohNode(iroh_dir.name)

    # ensure zero blobs
    blobs = node.blobs_list()
    assert len(blobs) == 0

    # create callback to get blobs and collection hash
    class AddCallback:
        collection_hash = None
        format = None
        blob_hashes = []

        def progress(self, progress_event):
            print(progress_event.type())
            if progress_event.type() == AddProgressType.ALL_DONE:
                all_done_event = progress_event.as_all_done()
                self.collection_hash = all_done_event.hash
                self.format = all_done_event.format
            if progress_event.type() == AddProgressType.ABORT:
                abort_event = progress_event.as_abort()
                raise Exception(abort_event.error)
            if progress_event.type() == AddProgressType.DONE:
                done_event = progress_event.as_done()
                print(done_event.hash.to_string())
                self.blob_hashes.append(done_event.hash)

    cb = AddCallback()
    tag = SetTagOption.auto()
    wrap = WrapOption.no_wrap()
    # add from path
    node.blobs_add_from_path(collection_dir.name, False, tag, wrap, cb)

    assert cb.collection_hash != None
    assert cb.format == BlobFormat.HASH_SEQ
 
    # list collections
    collections = node.blobs_list_collections()
    print("collection hash ", collections[0].hash.to_string())
    assert len(collections) == 1
    assert collections[0].hash.equal(cb.collection_hash)
    # should the blobs_count be 4?
    assert collections[0].total_blobs_count == 4
    # this always returns as None
    # assert collections[0].total_blobs_size == 300

    # list blobs
    collection_hashes = cb.blob_hashes
    collection_hashes.append(cb.collection_hash)
    got_hashes = node.blobs_list()
    for hash in got_hashes:
        blob = node.blobs_read_to_bytes(hash)
        print("hash ", hash.to_string(), " has size ", len(blob))
            
    hashes_exist(collection_hashes, got_hashes) 
    # collections also create a metadata hash that is not accounted for 
    # in the list of hashes
    assert len(collection_hashes)+1 == len(got_hashes)

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

    got_hashes = node.blobs_list()
    assert len(got_hashes) == num_blobs 
    hashes_exist(hashes, got_hashes)

    remove_hash = hashes.pop(0)
    node.blobs_delete_blob(remove_hash)

    got_hashes = node.blobs_list();
    assert len(got_hashes) == num_blobs - 1
    hashes_exist(hashes, got_hashes)

    for hash in got_hashes:
        if remove_hash.equal(hash):
            raise Exception("blob {} should have been removed", remove_hash)

def hashes_exist(expect, got):
    for hash in expect:
        exists = False
        for h in got:
            if h.equal(hash):
                exists = True
        if not exists:
            raise Exception("could not find ", hash, "in list")

# def test_download():
    # need to wait to refactor IrohNode to take an rpc port, or we remove rpc
    # ports from the iroh rpc in general 

