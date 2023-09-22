package main

import (
	"fmt"

	"github.com/n0-computer/iroh-ffi/iroh"
)

func main() {
	fmt.Printf("Booting...\n")
	node, err := iroh.NewIrohNode()
	if err != nil {
		panic(err)
	}

	peerID := node.PeerId()
	fmt.Printf("Hello, iroh %s from go!\n", peerID)

	conns, err := node.Connections()
	if err != nil {
		panic(err)
	}

	fmt.Printf("Got %d connections\n", len(conns))
	for _, conn := range conns {
		fmt.Printf("conn: %v\n", conn)
	}

	doc, err := node.CreateDoc()
	if err != nil {
		panic(err)
	}
	fmt.Printf("Created document %s\n", doc.Id())
	author, err := node.CreateAuthor()
	if err != nil {
		panic(err)
	}
	fmt.Printf("Created author %s\n", author.ToString())
	hash, err := doc.SetBytes(author, []byte("go"), []byte("says hello"))
	if err != nil {
		panic(err)
	}
	fmt.Printf("Inserted %s\n", hash.ToString())

	content, err := doc.GetContentBytes(hash)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Got content \"%s\"\n", string(content))

	hash, err = doc.SetBytes(author, []byte("another one"), []byte("says hello"))
	if err != nil {
		panic(err)
	}
	fmt.Printf("Inserted %s\n", hash.ToString())

	entries, err := doc.All()
	if err != nil {
		panic(err)
	}
	fmt.Printf("Got %d entries\n", len(entries))
	for _, entry := range entries {
		content, err := doc.GetContentBytes(entry.Hash())
		if err != nil {
			panic(err)
		}
		fmt.Printf("Entry: %s: \"%s\"\n", string(entry.Key()), string(content))
	}

	fmt.Printf("Goodbye!\n")
}
