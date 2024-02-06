package main

import (
	"crypto/rand"
	"io/ioutil"
	"os"
	"path/filepath"
	"testing"

	"github.com/n0-computer/iroh-ffi/iroh-go/iroh"

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

	// derp Url
	derpUrl := "https://example.com"

	// create a NodeAddr
	expectAddrs := []*iroh.SocketAddr{ipv4, ipv6}
	nodeAddrs := iroh.NewNodeAddr(nodeId, &derpUrl, expectAddrs)

	// test we have returned the expected addresses
	gotAddrs := nodeAddrs.DirectAddresses()
	for i := 0; i < len(expectAddrs); i++ {
		assert.True(t, gotAddrs[i].Equal(expectAddrs[i]))
		assert.True(t, expectAddrs[i].Equal(gotAddrs[i]))
	}

	assert.Equal(t, derpUrl, *nodeAddrs.DerpUrl())
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

/// TestQuery tests all the Query builders
func TestQuery(t *testing.T) {
	// all
	opts := iroh.QueryOptions{
		SortBy:    iroh.SortByKeyAuthor,
		Direction: iroh.SortDirectionAsc,
		Offset:    10,
		Limit:     10,
	}
	all := iroh.QueryAll(&opts)
	assert.Equal(t, opts.Offset, all.Offset())
	assert.Equal(t, opts.Limit, *all.Limit())

	// single_latest_per_key
	opts.Direction = iroh.SortDirectionDesc
	opts.Offset = 0
	opts.Limit = 0
	single_latest_per_key := iroh.QuerySingleLatestPerKey(&opts)
	assert.Equal(t, opts.Offset, single_latest_per_key.Offset())
	assert.Nil(t, single_latest_per_key.Limit())

	// author
	id, err := iroh.AuthorIdFromString("mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q")
	assert.Nil(t, err)

	opts.SortBy = iroh.SortByAuthorKey
	opts.Direction = iroh.SortDirectionAsc
	opts.Offset = 100
	opts.Limit = 0
	author := iroh.QueryAuthor(id, &opts)
	assert.Equal(t, opts.Offset, author.Offset())
	assert.Nil(t, author.Limit())

	// key_exact
	opts.SortBy = iroh.SortByKeyAuthor
	opts.Direction = iroh.SortDirectionDesc
	opts.Offset = 0
	opts.Limit = 10
	key_exact := iroh.QueryKeyExact(
		[]byte("key"),
		&opts,
	)
	assert.Equal(t, opts.Offset, key_exact.Offset())
	assert.Equal(t, opts.Limit, *key_exact.Limit())

	// key_prefix
	key_prefix := iroh.QueryKeyPrefix(
		[]byte("prefix"),
		&opts,
	)
	assert.Equal(t, opts.Offset, key_prefix.Offset())
	assert.Equal(t, opts.Limit, *key_prefix.Limit())
}

/// TestDocEntryBasics tests the basic flow from doc to entry
func TestDocEntryBasics(t *testing.T) {
	// create node
	dir, err := os.MkdirTemp("", "add_get_bytes")
	assert.Nil(t, err)

	defer os.RemoveAll(dir)

	node, err := iroh.NewIrohNode(dir)
	assert.Nil(t, err)

	// create doc and author
	doc, err := node.DocCreate()
	assert.Nil(t, err)
	author, err := node.AuthorCreate()
	assert.Nil(t, err)

	// create entry
	val := []byte("hello world!")
	key := []byte("foo")
	hash, err := doc.SetBytes(author, key, val)
	assert.Nil(t, err)

	// get entry
	maybe_entry, err := doc.GetExact(author, key, false)
	assert.NotNil(t, maybe_entry)
	entry := *maybe_entry
	assert.Nil(t, err)
	assert.True(t, hash.Equal(entry.ContentHash()))
	assert.Equal(t, uint64(len(val)), entry.ContentLen())
	got_val, err := entry.ContentBytes(doc)
	assert.Equal(t, val, got_val)
}

func TestDocImportExport(t *testing.T) {
	// Create temporary directory
	dir, err := ioutil.TempDir("", "test")
	assert.Nil(t, err)
	defer os.RemoveAll(dir)

	inRoot := filepath.Join(dir, "in")
	outRoot := filepath.Join(dir, "out")
	err = os.MkdirAll(inRoot, os.ModePerm)
	assert.Nil(t, err)
	err = os.MkdirAll(outRoot, os.ModePerm)
	assert.Nil(t, err)

	// Create file
	path := filepath.Join(inRoot, "test")
	size := 100
	bytes := make([]byte, size)
	_, err = rand.Read(bytes)
	assert.Nil(t, err)
	err = ioutil.WriteFile(path, bytes, 0644)
	assert.Nil(t, err)

	// Create node
	irohDir, err := ioutil.TempDir("", "iroh")
	assert.Nil(t, err)
	defer os.RemoveAll(irohDir)
	node, err := iroh.NewIrohNode(irohDir)
	assert.Nil(t, err)

	// Create doc and author
	doc, err := node.DocCreate()
	assert.Nil(t, err)
	author, err := node.AuthorCreate()
	assert.Nil(t, err)

	// Import entry
	key, err := iroh.PathToKey(path, nil, &inRoot)
	assert.Nil(t, err)

	err = doc.ImportFile(author, key, path, true, nil)
	assert.Nil(t, err)

	// Get entry
	maybe_entry, err := doc.GetExact(author, key, false)
	assert.Nil(t, err)
	entry := *maybe_entry

	// Export entry
	exportPath, err := iroh.KeyToPath(key, nil, &outRoot)
	assert.Nil(t, err)
	err = doc.ExportFile(entry, exportPath, nil)
	assert.Nil(t, err)

	// Read file
	gotBytes, err := ioutil.ReadFile(exportPath)
	assert.Nil(t, err)

	assert.Equal(t, bytes, gotBytes)
}
