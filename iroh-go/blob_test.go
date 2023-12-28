package main

import (
	"crypto/rand"
	"fmt"
	"os"
	"path"
	"strconv"
	"testing"

	"github.com/n0-computer/iroh-ffi/iroh-go/iroh"

	"github.com/stretchr/testify/assert"
)

// TestHash tests all functionality for the Hash struct
func TestHash(t *testing.T) {
	hashStr := "2kbxxbofqx5rau77wzafrj4yntjb4gn4olfpwxmv26js6dvhgjhq"
	hexStr := "d2837b85c585fb1053ffb64058a7986cd21e19bc72cafb5d95d7932f0ea7324f"
	bytes := []byte("\xd2\x83\x7b\x85\xc5\x85\xfb\x10\x53\xff\xb6\x40\x58\xa7\x98\x6c\xd2\x1e\x19\xbc\x72\xca\xfb\x5d\x95\xd7\x93\x2f\x0e\xa7\x32\x4f")

	// create hash from string
	hash, err := iroh.HashFromString(hashStr)
	assert.Nil(t, err)

	// test methods are as expected
	assert.Equal(t, hashStr, hash.ToString())
	assert.Equal(t, bytes, hash.ToBytes())
	assert.Equal(t, hexStr, hash.ToHex())

	// create hash from bytes
	hash0, err := iroh.HashFromBytes(bytes)
	assert.Nil(t, err)

	// test methods are as expected
	assert.Equal(t, hashStr, hash0.ToString())
	assert.Equal(t, bytes, hash0.ToBytes())
	assert.Equal(t, hexStr, hash0.ToHex())

	// test that the eq function works
	assert.True(t, hash.Equal(hash0))
	assert.True(t, hash0.Equal(hash))
}

// test functionality between adding as bytes and reading to bytes
func TestBlobAddGetBytes(t *testing.T) {
	// create node
	dir, err := os.MkdirTemp("", "add_get_bytes")
	assert.Nil(t, err)

	defer os.RemoveAll(dir)

	node, err := iroh.NewIrohNode(dir)
	assert.Nil(t, err)

	// create bytes
	var blobSize uint64 = 100
	bytes := randomBytes(t, blobSize)

	// add blob
	tag := iroh.SetTagOptionAuto()
	addOutcome, err := node.BlobsAddBytes(bytes, tag)
	assert.Nil(t, err)

	// check outcome info is as expected
	assert.Equal(t, iroh.BlobFormatRaw, addOutcome.Format)
	assert.Equal(t, addOutcome.Size, blobSize)

	// check we get the expected size from the hash
	hash := addOutcome.Hash

	gotSize, err := node.BlobsSize(hash)
	assert.Nil(t, err)
	assert.Equal(t, blobSize, gotSize)

	// get bytes
	gotBytes, err := node.BlobsReadToBytes(hash)
	assert.Nil(t, err)
	assert.Equal(t, blobSize, uint64(len(gotBytes)))
	assert.Equal(t, bytes, gotBytes)
}

type readWriteAddCallback struct {
	hash_ch   chan iroh.Hash
	format_ch chan iroh.BlobFormat
}

func (a readWriteAddCallback) Progress(event *iroh.AddProgress) *iroh.IrohError {
	if event.Type() == iroh.AddProgressTypeAllDone {
		all_done := event.AsAllDone()

		fmt.Println("all done event hash ", all_done.Hash, ", format ", all_done.Format)
		a.hash_ch <- *all_done.Hash
		a.format_ch <- all_done.Format
	} else if event.Type() == iroh.AddProgressTypeAbort {
		abort := event.AsAbort()
		// should be able to return the reason here
		fmt.Println("aborting add: ", abort.Error)
		return &iroh.IrohError{}
	}
	return nil
}

// test functionality between reading bytes from a path and writing bytes to
// a path
func TestBlobReadWritePath(t *testing.T) {
	irohDir, err := os.MkdirTemp("", "blob_read_write_path")
	assert.Nil(t, err)
	defer os.RemoveAll(irohDir)

	node, err := iroh.NewIrohNode(irohDir)
	assert.Nil(t, err)

	// create bytes
	var blobSize uint64 = 100
	bytes := randomBytes(t, blobSize)

	// write to file
	dir, err := os.MkdirTemp("", "blob_read_write_path_data")
	assert.Nil(t, err)
	data_path := path.Join(dir, "in")
	err = os.WriteFile(data_path, bytes, 0644)
	assert.Nil(t, err)

	// add blob
	tag := iroh.SetTagOptionAuto()
	wrap := iroh.WrapOptionNoWrap()

	hash_ch := make(chan iroh.Hash)
	format_ch := make(chan iroh.BlobFormat)
	cb := readWriteAddCallback{hash_ch, format_ch}
	go node.BlobsAddFromPath(data_path, false, tag, wrap, cb)

	hash := <-hash_ch
	format := <-format_ch
	// check outcome info is as expected
	assert.Equal(t, iroh.BlobFormatRaw, format)
	assert.NotNil(t, hash)

	// check we get the expected size from the hash
	gotSize, err := node.BlobsSize(&hash)
	assert.Nil(t, err)
	assert.Equal(t, blobSize, gotSize)

	// get bytes
	gotBytes, err := node.BlobsReadToBytes(&hash)
	assert.Nil(t, err)
	fmt.Println("BlobsReadToBytes ", gotBytes)
	assert.Equal(t, blobSize, uint64(len(gotBytes)))
	assert.Equal(t, bytes, gotBytes)

	// write to file
	outPath := path.Join(dir, "out")
	node.BlobsWriteToPath(&hash, outPath)

	// read file
	gotBytes, err = os.ReadFile(outPath)
	assert.Nil(t, err)
	fmt.Println("BlobsWriteToPath ", gotBytes)
	assert.Equal(t, blobSize, uint64(len(gotBytes)))
	assert.Equal(t, bytes, gotBytes)
}

type collectionsAddCallback struct {
	collectionHashCh chan iroh.Hash
	formatCh         chan iroh.BlobFormat
	hashes           *[]iroh.Hash
}

func (a collectionsAddCallback) Progress(event *iroh.AddProgress) *iroh.IrohError {
	if event.Type() == iroh.AddProgressTypeAllDone {
		all_done := event.AsAllDone()

		fmt.Println("all done event hash ", all_done.Hash, ", format ", all_done.Format)
		a.collectionHashCh <- *all_done.Hash
		a.formatCh <- all_done.Format
	} else if event.Type() == iroh.AddProgressTypeAbort {
		abort := event.AsAbort()
		// should be able to return the reason here
		fmt.Println("aborting add: ", abort.Error)
		return &iroh.IrohError{}
	} else if event.Type() == iroh.AddProgressTypeDone {
		done := event.AsDone()
		fmt.Printf("hash %s\n", done.Hash.ToString())
		fmt.Printf("before hashes len %d\n", len(*a.hashes))
		*a.hashes = append(*a.hashes, *done.Hash)
	}
	return nil
}

// TestBlobCollections tests the functionality of creating a collection
// via the `IrohNode.BlobsAddFromPath` method and using a `AddCallback`
// interface
func TestBlobCollections(t *testing.T) {
	collectionDir, err := os.MkdirTemp("", "blob_collections_data")
	assert.Nil(t, err)
	defer os.RemoveAll(collectionDir)

	numFiles := 3
	var blobSize uint64 = 100
	for i := 0; i < numFiles; i++ {
		path := path.Join(collectionDir, strconv.Itoa(i))
		bytes := randomBytes(t, blobSize)
		err := os.WriteFile(path, bytes, 0644)
		assert.Nil(t, err)
	}

	// make node
	irohDir, err := os.MkdirTemp("", "blob_collections_iroh_node")
	assert.Nil(t, err)
	defer os.RemoveAll(irohDir)

	node, err := iroh.NewIrohNode(irohDir)
	assert.Nil(t, err)

	// ensure zero blobs
	blobs, err := node.BlobsList()
	assert.Nil(t, err)
	assert.Equal(t, 0, len(blobs))

	collectionHashCh := make(chan iroh.Hash)
	formatCh := make(chan iroh.BlobFormat)
	hashes := []iroh.Hash{}

	cb := collectionsAddCallback{
		collectionHashCh: collectionHashCh,
		formatCh:         formatCh,
		hashes:           &hashes,
	}
	tag := iroh.SetTagOptionAuto()
	wrap := iroh.WrapOptionNoWrap()
	// add from path
	go node.BlobsAddFromPath(collectionDir, false, tag, wrap, cb)

	collectionHash := <-collectionHashCh
	format := <-formatCh

	assert.NotNil(t, collectionHash)
	assert.Equal(t, iroh.BlobFormatHashSeq, format)

	// list collections
	collections, err := node.BlobsListCollections()
	assert.Nil(t, err)

	fmt.Println("collection hash ", collections[0].Hash.ToString())
	assert.Equal(t, 1, len(collections))
	assert.True(t, collections[0].Hash.Equal(&collectionHash))
	// should the blobs_count be 4?
	assert.Equal(t, uint64(4), *collections[0].TotalBlobsCount)
	// this returns a size of nil
	// assert.Equal(t, uint64(300), *collections[0].TotalBlobsSize)

	// list blobs
	gotHashes, err := node.BlobsList()
	assert.Nil(t, err)
	for _, hash := range gotHashes {
		blob, err := node.BlobsReadToBytes(hash)
		assert.Nil(t, err)
		fmt.Println("hash ", hash.ToString(), " has size ", len(blob))
	}

	// check that all hashes exist
	collectionHashes := hashes
	collectionHashes = append(collectionHashes, collectionHash)
	fmt.Println("collection Hashes:")
	for _, hash := range collectionHashes {
		blob, err := node.BlobsReadToBytes(&hash)
		assert.Nil(t, err)
		fmt.Println("hash ", hash.ToString(), " has size ", len(blob))
	}
	hashesExist(collectionHashes, gotHashes)
	// the collection would have also created a meta data blob that
	// is not accounted for when we pull the hashes from the collection
	// as it was being made
	assert.Equal(t, len(collectionHashes)+1, len(gotHashes))
}

// TestListAndDelete tests the functionality of listing and removing blobs
func TestListAndDelete(t *testing.T) {
	// make node
	irohDir, err := os.MkdirTemp("", "blob_collections_iroh_node")
	assert.Nil(t, err)
	defer os.RemoveAll(irohDir)

	node, err := iroh.NewIrohNode(irohDir)
	assert.Nil(t, err)

	// create bytes
	var blobSize uint64 = 100
	blobs := [][]byte{}
	numBlobs := 3

	for i := 0; i < numBlobs; i++ {
		bytes := randomBytes(t, blobSize)
		blobs = append(blobs, bytes)
	}

	hashes := []iroh.Hash{}
	for _, blob := range blobs {
		output, err := node.BlobsAddBytes(blob, iroh.SetTagOptionAuto())
		assert.Nil(t, err)
		hashes = append(hashes, *output.Hash)
	}

	list, err := node.BlobsList()
	assert.Nil(t, err)
	assert.Equal(t, numBlobs, len(list))
	hashesExist(hashes, list)

	removeHash := hashes[0]
	hashes = hashes[1:]
	node.BlobsDeleteBlob(&removeHash)

	list, err = node.BlobsList()
	assert.Nil(t, err)
	assert.Equal(t, numBlobs-1, len(list))
	hashesExist(hashes, list)

	for _, hash := range list {
		if removeHash.Equal(hash) {
			panic(fmt.Sprintf("blinob %s should have been removed", removeHash.ToString()))
		}
	}
}

func hashesExist(expect []iroh.Hash, got []*iroh.Hash) {
	for _, hash := range expect {
		exists := false
		for _, h := range got {
			if h.Equal(&hash) {
				exists = true
			}
		}
		if !exists {
			panic(fmt.Sprintf("could not find %s in list", hash.ToString()))
		}
	}
}

/// TestDownload tests the blobs download functionality
func TestDownload(t *testing.T) {
	t.Skip()
	// need to wait to refactor IrohNode to take an rpc port, or we remove rpc
	// ports from the iroh rpc in general
}

func randomBytes(t *testing.T, size uint64) []byte {
	bytes := make([]byte, size)
	_, err := rand.Read(bytes)
	assert.Nil(t, err)
	return bytes
}
