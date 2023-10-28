package main

import (
	"testing"

	"github.com/n0-computer/iroh-ffi/iroh"
	"github.com/stretchr/testify/assert"
)

/// Test all PeerAddr functionality
func TestPeerAddr(t *testing.T) {
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

	// create a PeerAddr
	expectAddrs := []*iroh.SocketAddr{ipv4, ipv6}
	peerAddrs := iroh.NewPeerAddr(nodeId, &derpRegion, expectAddrs)

	// test we have returned the expected addresses
	gotAddrs := peerAddrs.DirectAddresses()
	for i := 0; i < len(expectAddrs); i++ {
		assert.True(t, gotAddrs[i].Equal(expectAddrs[i]))
		assert.True(t, expectAddrs[i].Equal(gotAddrs[i]))
	}

	assert.Equal(t, peerAddrs.DerpRegion(), &derpRegion)
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
	docTicketStr := "ljapn77ljjzwrtxh4b35xg57gfvcrvey6ofrulgzuddnohwc2qnqcicshr4znowxoqsosz4gz55hebirkm32lncwltjfkbva6kl3denf5iaqcbiajjeteswek4ambkabzpcfoajganyabbz2zplaaaaaaaaaagrjyvlqcjqdoaaioowl2ygi2likyov62rofk4asma3qacdtvs6wrg7f7hkxlg3mlrkx"
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

/// Test all GetFilter functionality
func TestGetFilter(t *testing.T) {
	// all
	all := iroh.GetFilterAll()

	// key
	key := iroh.GetFilterKey([]byte("key"))
	key0 := iroh.GetFilterKey([]byte("key"))
	assert.False(t, all.Equal(key))
	assert.True(t, key0.Equal(key))

	// prefix
	prefix := iroh.GetFilterPrefix([]byte("prefix"))
	prefix0 := iroh.GetFilterPrefix([]byte("prefix"))
	assert.False(t, key.Equal(prefix))
	assert.True(t, prefix.Equal(prefix0))

	// author
	authorStr := "mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q"
	a, err := iroh.AuthorIdFromString(authorStr)
	if err != nil {
		panic(err)
	}
	author := iroh.GetFilterAuthor(a)
	author0 := iroh.GetFilterAuthor(a)
	assert.False(t, prefix.Equal(author))
	assert.True(t, author.Equal(author0))

	// author&prefix
	authorPrefix := iroh.GetFilterAuthorPrefix(a, []byte("prefix"))
	authorPrefix0 := iroh.GetFilterAuthorPrefix(a, []byte("prefix"))
	assert.False(t, author.Equal(authorPrefix))
	assert.True(t, authorPrefix.Equal(authorPrefix0))
}
