package main

import (
	"testing"

	"github.com/n0-computer/iroh-ffi/iroh"
	"github.com/stretchr/testify/assert"
)

/// Test all NodeAddr functionality
func TestNodeAddr(t *testing.T) {
	// create a nodeId
	keyStr := "ki6htfv2252cj2lhq3hxu4qfcfjtpjnukzonevigudzjpmmruxva"
	nodeId, err := iroh.PublicKeyFromString(keyStr)
	if err != nil {
		panic(err)
	}

	// create socketaddrs
	ipv4Ip, err := iroh.Ipv4AddrFromString("127.0.0.1")
	if err != nil {
		panic(err)
	}
	ipv6Ip, err := iroh.Ipv6AddrFromString("::1")
	if err != nil {
		panic(err)
	}

	var port uint16 = 3000

	// create socket addrs
	ipv4 := iroh.SocketAddrFromIpv4(ipv4Ip, port)
	ipv6 := iroh.SocketAddrFromIpv6(ipv6Ip, port)

	// derp region
	var derpRegion uint16 = 1

	// create a NodeAddr
	expectAddrs := []*iroh.SocketAddr{ipv4, ipv6}
	nodeAddrs := iroh.NewNodeAddr(nodeId, &derpRegion, expectAddrs)

	// test we have returned the expected addresses
	gotAddrs := nodeAddrs.DirectAddresses()
	for i := 0; i < len(expectAddrs); i++ {
		assert.True(t, gotAddrs[i].Equal(expectAddrs[i]))
		assert.True(t, expectAddrs[i].Equal(gotAddrs[i]))
	}

	assert.Equal(t, nodeAddrs.DerpRegion(), &derpRegion)
}

/// Test all NamespaceId functionality
func TestNamespaceId(t *testing.T) {
	// create id from string
	namespaceStr := "mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q"
	namespace, err := iroh.NamespaceIdFromString(namespaceStr)
	if err != nil {
		panic(err)
	}

	// call ToString, ensure Equal
	assert.Equal(t, namespace.ToString(), namespaceStr)
	// create another id, same string
	namespace0, err := iroh.NamespaceIdFromString(namespaceStr)
	if err != nil {
		panic(err)
	}

	// ensure Equal
	assert.True(t, namespace.Equal(namespace0))
	assert.True(t, namespace0.Equal(namespace))
}

/// Test all AuthorId functionality
func TestAuthorId(t *testing.T) {
	// create id from string
	authorStr := "mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q"
	author, err := iroh.AuthorIdFromString(authorStr)
	if err != nil {
		panic(err)
	}

	// call ToString, ensure Equal
	assert.Equal(t, author.ToString(), authorStr)
	// create another id, same string
	author0, err := iroh.AuthorIdFromString(authorStr)
	if err != nil {
		panic(err)
	}

	// ensure Equal
	assert.True(t, author.Equal(author0))
	assert.True(t, author0.Equal(author))
}

/// Test all DocTicket functionality
func TestDocTicket(t *testing.T) {
	// create id from string
	docTicketStr := "docaaqjjfgbzx2ry4zpaoujdppvqktgvfvpxgqubkghiialqovv7z4wosqbebpvjjp2tywajvg6unjza6dnugkalg4srmwkcucmhka7mgy4r3aa4aibayaeusjsjlcfoagavaa4xrcxaetag4aaq45mxvqaaaaaaaaadiu4kvybeybxaaehhlf5mdenfufmhk7nixcvoajganyabbz2zplgbno2vsnuvtkpyvlqcjqdoaaioowl22k3fc26qjx4ot6fk4"
	docTicket, err := iroh.DocTicketFromString(docTicketStr)
	if err != nil {
		panic(err)
	}

	// call ToString, ensure Equal
	assert.Equal(t, docTicket.ToString(), docTicketStr)
	// create another ticket, same string
	docTicket0, err := iroh.DocTicketFromString(docTicketStr)
	if err != nil {
		panic(err)
	}

	// ensure Equal
	assert.True(t, docTicket.Equal(docTicket0))
	assert.True(t, docTicket0.Equal(docTicket))
}

/// TestQuery tests all the Query builders
func TestQuery(t *testing.T) {
	// all
	var offset uint64 = 10
	var limit uint64 = 10
	all := iroh.QueryAll(iroh.SortByKeyAuthor, iroh.SortDirectionAsc, &offset, &limit)
	assert.Equal(t, offset, all.Offset())
	assert.Equal(t, limit, *all.Limit())

	// single_latest_per_key
	single_latest_per_key := iroh.QuerySingleLatestPerKey(iroh.SortDirectionDesc, nil, nil)
	offset = 0
	assert.Equal(t, offset, single_latest_per_key.Offset())
	assert.Nil(t, single_latest_per_key.Limit())

	// author
	id, err := iroh.AuthorIdFromString("mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q")
	assert.Nil(t, err)

	offset = 100
	limit = 100
	author := iroh.QueryAuthor(id, iroh.SortByAuthorKey,
		iroh.SortDirectionAsc,
		&offset,
		nil,
	)
	assert.Equal(t, offset, author.Offset())
	assert.Nil(t, author.Limit())

	// key_exact
	key_exact := iroh.QueryKeyExact(
		[]byte("key"),
		iroh.SortByKeyAuthor,
		iroh.SortDirectionDesc,
		nil,
		&limit,
	)
	offset = 0
	assert.Equal(t, offset, key_exact.Offset())
	assert.Equal(t, limit, *key_exact.Limit())

	// key_prefix
	key_prefix := iroh.QueryKeyPrefix(
		[]byte("prefix"),
		iroh.SortByKeyAuthor,
		iroh.SortDirectionDesc,
		nil,
		&limit,
	)
	assert.Equal(t, offset, key_prefix.Offset())
	assert.Equal(t, limit, *key_prefix.Limit())
}
