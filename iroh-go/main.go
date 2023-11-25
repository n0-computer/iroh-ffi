package main

import (
	"fmt"
	"os"

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

	doc, err := node.DocCreate()
	if err != nil {
		panic(err)
	}
	fmt.Printf("Created document %s\n", doc.Id().ToString())
	author, err := node.AuthorCreate()
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

	entries, err := doc.GetMany(iroh.QueryAll(nil))
	if err != nil {
		panic(err)
	}
	fmt.Printf("Got %d entries\n", len(entries))
	for _, entry := range entries {
		content, err := doc.ReadToBytes(entry)
		if err != nil {
			panic(err)
		}
		fmt.Printf("Entry: %s: \"%s\"\n", string(entry.Key()), string(content))
	}

	doc, err = node.DocCreate()
	if err != nil {
		panic(err)
	}

	fmt.Printf("Created second document %s\n", doc.Id().ToString())

	docs, err := node.DocList()
	if err != nil {
		panic(err)
	}

	fmt.Printf("Listing all %d documents:\n", len(docs))
	for _, doc_and_capability := range docs {
		fmt.Printf("\t%s\n", doc_and_capability.Namespace.ToString())
	}

	fmt.Printf("Goodbye!\n")
}
