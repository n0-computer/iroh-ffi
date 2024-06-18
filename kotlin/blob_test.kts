// tests that correspond to the `src/doc.rs` rust api

import iroh.*
import kotlin.random.Random

fun generateRandomByteArray(size: Int): ByteArray {
    val byteArray = ByteArray(size)
    Random.nextBytes(byteArray)
    return byteArray
}

// Hash
fun testHash() {
    val hashStr = "2kbxxbofqx5rau77wzafrj4yntjb4gn4olfpwxmv26js6dvhgjhq"
    val hexStr = "d2837b85c585fb1053ffb64058a7986cd21e19bc72cafb5d95d7932f0ea7324f"
    val bytes =
        ubyteArrayOf(
            0xd2u,
            0x83u,
            0x7bu,
            0x85u,
            0xc5u,
            0x85u,
            0xfbu,
            0x10u,
            0x53u,
            0xffu,
            0xb6u,
            0x40u,
            0x58u,
            0xa7u,
            0x98u,
            0x6cu,
            0xd2u,
            0x1eu,
            0x19u,
            0xbcu,
            0x72u,
            0xcau,
            0xfbu,
            0x5du,
            0x95u,
            0xd7u,
            0x93u,
            0x2fu,
            0x0eu,
            0xa7u,
            0x32u,
            0x4fu,
        ).toByteArray()

    // create hash from string
    val hash = Hash.fromString(hashStr)

    // test methods are as expected
    assert(hash.toString() == hashStr)
    assert(hash.toBytes() contentEquals bytes)
    assert(hash.toHex() == hexStr)

    // create hash from bytes
    val hash0 = Hash.fromBytes(bytes)

    // test methods are as expected
    assert(hash0.toString() == hashStr)
    assert(hash0.toBytes() contentEquals bytes)
    assert(hash0.toHex() == hexStr)

    // test that the eq function works
    assert(hash.equal(hash0))
    assert(hash0.equal(hash))
}
testHash()

// test functionality between adding as bytes and reading to bytes
fun testAddBytes() {
    // create node
    val irohDir = kotlin.io.path.createTempDirectory("doc-test")
    val node = IrohNode(irohDir.toString())

    // create bytes
    val blobSize = 100
    val bytes = generateRandomByteArray(blobSize)

    // add blob
    val addOutcome = node.blobsAddBytes(bytes)

    // check outcome info is as expected
    assert(addOutcome.format == BlobFormat.RAW)
    assert(addOutcome.size == blobSize.toULong())

    // check we get the expected size from the hash
    val hash = addOutcome.hash
    val gotSize = node.blobsSize(hash)
    assert(gotSize == blobSize.toULong())

    // get bytes
    val gotBytes = node.blobsReadToBytes(hash)
    assert(gotBytes.size == blobSize)
    assert(gotBytes contentEquals bytes)
}
testAddBytes()

// test functionality between reading bytes from a path and writing bytes to a path
fun testReadBytesPath() {
    val irohDir = kotlin.io.path.createTempDirectory("doc-test-read-bytes")
    val node = IrohNode(irohDir.toString())

    // create bytes
    val blobSize = 100
    val bytes = generateRandomByteArray(blobSize)

    // write to file
    val dir = kotlin.io.path.createTempDirectory("doc-test-read-bytes-r-file")
    val path = dir.toString() + "in"
    java.io.File(path).writeBytes(bytes)

    // add blob
    val tag = SetTagOption.auto()
    val wrap = WrapOption.noWrap()

    class Handler : AddCallback {
        var hash: Hash? = null
        var format: BlobFormat? = null

        override fun progress(progress: AddProgress) {
            println(progress.type())
            if (progress.type() == AddProgressType.ALL_DONE) {
                val event = progress.asAllDone()!!
                this.hash = event.hash
                println(event.hash)
                println(event.format)
                this.format = event.format
            }
            if (progress.type() == AddProgressType.ABORT) {
                val event = progress.asAbort()!!
                throw Exception(event.error)
            }
        }
    }
    val cb = Handler()
    node.blobsAddFromPath(path, false, tag, wrap, cb)

    // check outcome info is as expected
    assert(cb.format == BlobFormat.RAW)
    assert(cb.hash != null)

    // check we get the expected size from the hash
    val gotSize = node.blobsSize(cb.hash!!)
    assert(gotSize == blobSize.toULong())

    // get bytes
    val gotBytes = node.blobsReadToBytes(cb.hash!!)
    assert(gotBytes.size == blobSize)
    assert(gotBytes contentEquals bytes)

    // write to file
    val outPath = dir.toString() + "out"
    node.blobsWriteToPath(cb.hash!!, outPath)

    // open file
    val gotBytesFile = java.io.File(outPath).readBytes()
    assert(gotBytesFile.size == blobSize)
    assert(gotBytesFile contentEquals bytes)
}
testReadBytesPath()

// Collections
// {
//     collection_dir = tempfile.TemporaryDirectory()
//     num_files = 3
//     blob_size = 100
//     for i in range(num_files):
//         path = os.path.join(collection_dir.name, str(i))
//         bytes = bytearray(map(random.getrandbits,(8,)*blob_size))
//         file = open(path, "wb")
//         file.write(bytes)
//         file.close()
//     print(collection_dir.__sizeof__())

//     // make node
//     iroh_dir = tempfile.TemporaryDirectory()
//     node = IrohNode(iroh_dir.name)

//     // ensure zero blobs
//     blobs = node.blobs_list()
//     assert len(blobs) == 0

//     // create callback to get blobs and collection hash
//     class AddCallback:
//         collection_hash = None
//         format = None
//         blob_hashes = []

//         def progress(self, progress_event):
//             print(progress_event.type())
//             if progress_event.type() == AddProgressType.ALL_DONE:
//                 all_done_event = progress_event.as_all_done()
//                 self.collection_hash = all_done_event.hash
//                 self.format = all_done_event.format
//             if progress_event.type() == AddProgressType.ABORT:
//                 abort_event = progress_event.as_abort()
//                 raise Exception(abort_event.error)
//             if progress_event.type() == AddProgressType.DONE:
//                 done_event = progress_event.as_done()
//                 print(done_event.hash)
//                 self.blob_hashes.append(done_event.hash)

//     cb = AddCallback()
//     tag = SetTagOption.auto()
//     wrap = WrapOption.no_wrap()
//     // add from path
//     node.blobs_add_from_path(collection_dir.name, False, tag, wrap, cb)

//     assert cb.collection_hash != None
//     assert cb.format == BlobFormat.HASH_SEQ

//     // list collections
//     collections = node.blobs_list_collections()
//     print("collection hash ", collections[0].hash)
//     assert len(collections) == 1
//     assert collections[0].hash.equal(cb.collection_hash)
//     // should the blobs_count be 4?
//     assert collections[0].total_blobs_count == 4
//     // this always returns as None
//     // assert collections[0].total_blobs_size == 300

//     // list blobs
//     collection_hashes = cb.blob_hashes
//     collection_hashes.append(cb.collection_hash)
//     got_hashes = node.blobs_list()
//     for hash in got_hashes:
//         blob = node.blobs_read_to_bytes(hash)
//         print("hash ", hash, " has size ", len(blob))

//     hashes_exist(collection_hashes, got_hashes)
//     // collections also create a metadata hash that is not accounted for
//     // in the list of hashes
//     assert len(collection_hashes)+1 == len(got_hashes)
// }

// List and delete
// {
//     iroh_dir = tempfile.TemporaryDirectory()
//     opts = NodeOptions(gc_interval_millis=100)
//     node = IrohNode.with_options(iroh_dir.name, opts)
//
//     // create bytes
//     blob_size = 100
//     blobs = []
//     num_blobs = 3;

//     for x in range(num_blobs):
//         print(x)
//         bytes = bytearray(map(random.getrandbits,(8,)*blob_size))
//         blobs.append(bytes)

//     hashes = []
//     tags = []
//     for blob in blobs:
//         output = node.blobs_add_bytes(blob)
//         hashes.append(output.hash)
//         tags.append(output.tag)

//     got_hashes = node.blobs_list()
//     assert len(got_hashes) == num_blobs
//     hashes_exist(hashes, got_hashes)

//     remove_hash = hashes.pop(0)
//     remove_tag = tags.pop(0)
//     // delete the tag for the first blob
//     node.tags_delete(remove_tag)
//     // wait for GC to clear the blob
//     time.sleep(0.25)

//     got_hashes = node.blobs_list();
//     assert len(got_hashes) == num_blobs - 1
//     hashes_exist(hashes, got_hashes)

//     for hash in got_hashes:
//         if remove_hash.equal(hash):
//             raise Exception("blob {} should have been removed", remove_hash)
// def hashes_exist(expect, got):
//     for hash in expect:
//         exists = False
//         for h in got:
//             if h.equal(hash):
//                 exists = True
//         if not exists:
//             raise Exception("could not find ", hash, "in list")
// }
