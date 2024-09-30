// tests that correspond to the `src/doc.rs` rust api
import iroh.*
import kotlinx.coroutines.runBlocking

// Node addr
runBlocking {
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

// Author Id
runBlocking {
    // create id from string
    val authorStr = "mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q"
    val author = AuthorId.fromString(authorStr)

    // call to_string, ensure equal
    assert(author.toString() == authorStr)

    // create another id, same string
    val author0 = AuthorId.fromString(authorStr)

    // ensure equal
    assert(author.equal(author0))
    assert(author0.equal(author))
}

// Query
runBlocking {
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
    val author = Query.author(AuthorId.fromString("mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q"), opts)
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

// Doc entry basics
runBlocking {
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
    val gotVal = entry.contentBytes(doc)
    assert(v contentEquals gotVal)
}
