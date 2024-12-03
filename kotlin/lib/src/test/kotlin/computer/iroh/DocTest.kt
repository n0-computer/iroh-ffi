package computer.iroh

import kotlinx.coroutines.test.runTest
import kotlin.test.Test

class DocTest {
    @Test fun nodeAddr() =
        runTest {
            // create a node_id
            val keyStr = "ki6htfv2252cj2lhq3hxu4qfcfjtpjnukzonevigudzjpmmruxva"
            val nodeId = PublicKey.fromString(keyStr)

            // create socketaddrs
            val ipv4 = "127.0.0.1:3000"
            val ipv6 = "::1:3000"

            // relay url
            val relayUrl = "https://example.com"

            // create a NodeAddr
            val expectAddrs = listOf(ipv4, ipv6)
            val nodeAddr = NodeAddr(nodeId, relayUrl, expectAddrs)

            // test we have returned the expected addresses
            val gotAddrs = nodeAddr.directAddresses()
            gotAddrs.zip(expectAddrs) { got, ex -> assert(got == ex) }
            assert(relayUrl == nodeAddr.relayUrl())
        }

    @Test fun authorId() =
        runTest {
            // create id from string
            val authorStr = "7db06b57aac9b3640961d281239c8f23487ac7f7265da21607c5612d3527a254"
            val author = AuthorId.fromString(authorStr)

            // call to_string, ensure equal
            assert(author.toString() == authorStr)

            // create another id, same string
            val author0 = AuthorId.fromString(authorStr)

            // ensure equal
            assert(author.equal(author0))
            assert(author0.equal(author))
        }

    @Test fun query() =
        runTest {
            var opts = QueryOptions(SortBy.KEY_AUTHOR, SortDirection.ASC, 10u, 10u)

            // all
            val all = Query.all(opts)
            assert(10UL == all.offset())
            assert(10UL == all.limit())

            // single_latest_per_key
            opts.direction = SortDirection.DESC
            opts.limit = 0u
            opts.offset = 0u
            val singleLatestPerKey = Query.singleLatestPerKey(opts)
            assert(0UL == singleLatestPerKey.offset())
            assert(null == singleLatestPerKey.limit())

            // author
            opts.direction = SortDirection.ASC
            opts.offset = 100u
            val author = Query.author(AuthorId.fromString("7db06b57aac9b3640961d281239c8f23487ac7f7265da21607c5612d3527a254"), opts)
            assert(100UL == author.offset())
            assert(null == author.limit())

            // key_exact
            opts.sortBy = SortBy.KEY_AUTHOR
            opts.direction = SortDirection.DESC
            opts.offset = 0u
            opts.limit = 100u
            val keyExact = Query.keyExact("key".toByteArray(), opts)
            assert(0UL == keyExact.offset())
            assert(100UL == keyExact.limit())

            // key_prefix
            val keyPrefix = Query.keyPrefix("prefix".toByteArray(), opts)
            assert(0UL == keyPrefix.offset())
            assert(100UL == keyPrefix.limit())
        }

    @Test fun docEntryBasics() =
        runTest {
            // create node
            val irohDir = kotlin.io.path.createTempDirectory("doc-test")
            val options = NodeOptions()
            options.enableDocs = true
            val node = Iroh.persistentWithOptions(irohDir.toString(), options)

            // create doc and author
            val doc = node.docs().create()
            val author = node.authors().create()

            // create entry
            val v = "hello world!".toByteArray()
            val key = "foo".toByteArray()
            val hash = doc.setBytes(author, key, v)

            // get entry
            val query = Query.authorKeyExact(author, key)
            val entry = doc.getOne(query)!!
            assert(hash.equal(entry.contentHash()))
            assert(v.size.toULong() == entry.contentLen())
            val gotVal: ByteArray =
                try {
                    node.blobs().readToBytes(entry.contentHash())
                } catch (e: IrohException) {
                    println("failed content bytes ${e.message}")
                    throw e
                }

            assert(v contentEquals gotVal)
            node.node().shutdown()
        }
}
