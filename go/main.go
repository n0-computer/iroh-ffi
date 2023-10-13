package main

import (
	"bufio"
	"fmt"
	"os"
	"strings"

	"github.com/n0-computer/iroh-ffi/iroh"
)

func main() {
	fmt.Printf("Booting...\n")
	nodeDir := "./iroh-node-go"
	if err := os.Mkdir(nodeDir, os.ModePerm); err != nil {
		panic(err)
	}
	node, err := iroh.NewIrohNode(nodeDir)
	if err != nil {
		panic(err)
	}

	nodeID := node.NodeId()
	fmt.Printf("Hello, iroh %s from go!\n", nodeID)

	conns, err := node.Connections()
	if err != nil {
		panic(err)
	}

	fmt.Printf("Got %d connections\n", len(conns))
	for _, conn := range conns {
		fmt.Printf("conn: %v\n", conn)
	}

	doc, err := node.DocNew()
	if err != nil {
		panic(err)
	}
	fmt.Printf("Created document %s\n", doc.Id())
	author, err := node.AuthorNew()
	if err != nil {
		panic(err)
	}
	fmt.Printf("Created author %s\n", author.ToString())
	hash, err := doc.SetBytes(author, []byte("go"), []byte("says hello"))
	if err != nil {
		panic(err)
	}
	fmt.Printf("Inserted %s\n", hash.ToString())

	// content, err := doc.GetContentBytes(entry)
	// if err != nil {
	// 	panic(err)
	// }
	// fmt.Printf("Got content \"%s\"\n", string(content))

	hash, err = doc.SetBytes(author, []byte("another one"), []byte("says hello"))
	if err != nil {
		panic(err)
	}
	fmt.Printf("Inserted %s\n", hash.ToString())

	entries, err := doc.Keys()
	if err != nil {
		panic(err)
	}
	fmt.Printf("Got %d entries\n", len(entries))
	for _, entry := range entries {
		content, err := doc.GetContentBytes(entry)
		if err != nil {
			panic(err)
		}
		fmt.Printf("Entry: %s: \"%s\"\n", string(entry.Key()), string(content))
	}

	doc, err = node.DocNew()
	if err != nil {
		panic(err)
	}

	fmt.Printf("Created second document %s\n", doc.Id())

	docs, err := node.DocList()
	if err != nil {
		panic(err)
	}

	fmt.Printf("Listing all %d documents:\n", len(docs))
	for _, doc_id := range docs {
		fmt.Printf("\t%s\n", doc_id.ToString())
	}

	incomplete_blobs, err := node.BlobListIncomplete()
	if err != nil {
		panic(err)
	}

	if len(incomplete_blobs) != 0 {
		fmt.Printf("Unexpected incomplete blobs:\n")
		for _, blob := range incomplete_blobs {
			fmt.Printf("\thash: %s expected size: %d size: %d", blob.Hash.ToString(), blob.ExpectedSize, blob.Size)
		}
	}

	// ensure BlobValidate executes
	// TODO: unimplemented in v0.7.0
	// invalid_blobs, err := node.BlobValidate(false)
	// if err != nil {
	// 	panic(err)
	// }

	// if len(invalid_blobs) != 0 {
	// 	fmt.Printf("Unexpected invalid blobs:\n")
	// 	for _, blob := range invalid_blobs {
	// 		fmt.Printf("\thash: %s name: %d size: %d", blob.Hash.ToString(), blob.Name, blob.Size)
	// 	}
	// }

	reader := bufio.NewReader(os.Stdin)
	fmt.Printf("\nSupply a path to add files to the blob store: ")
	text, err := reader.ReadString('\n')
	if err != nil {
		panic(err)
	}
	text = strings.TrimSpace(text)
	fmt.Printf("\nAdding %s to the blob store...\n", text)
	blobs, err := node.BlobAdd(text, false, nil, false, nil)
	if err != nil {
		panic(err)
	}
	for _, blob := range blobs {
		fmt.Printf("\tblob %s, hash %s, size %d\n", blob.Name, blob.Hash.ToString(), blob.Size)
	}

	reader = bufio.NewReader(os.Stdin)
	fmt.Printf("\nSupply a path to add files to the blob store: ")
	text, err = reader.ReadString('\n')
	if err != nil {
		panic(err)
	}
	text = strings.TrimSpace(text)
	fmt.Printf("\nAdding %s to the blob store...\n", text)
	blobs, err = node.BlobAdd(text, false, nil, false, nil)
	if err != nil {
		panic(err)
	}
	for _, blob := range blobs {
		fmt.Printf("\tblob %s, hash %s, size %d\n", blob.Name, blob.Hash.ToString(), blob.Size)
	}

	fmt.Printf("Goodbye!\n")
}
