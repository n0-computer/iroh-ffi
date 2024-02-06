package main

import (
	"fmt"
	"os"
	"testing"

	"github.com/n0-computer/iroh-ffi/iroh-go/iroh"

	"github.com/stretchr/testify/assert"
)

func TestBasicSync(t *testing.T) {
	// Create node_0
	irohDir0, err := os.MkdirTemp("", "iroh")
	assert.Nil(t, err)
	defer os.RemoveAll(irohDir0)

	node0, err := iroh.NewIrohNode(irohDir0)
	assert.Nil(t, err)

	// Create node_1
	irohDir1, err := os.MkdirTemp("", "iroh")
	assert.Nil(t, err)
	defer os.RemoveAll(irohDir1)

	node1, err := iroh.NewIrohNode(irohDir1)
	assert.Nil(t, err)

	// Create doc on node_0
	doc0, err := node0.DocCreate()
	assert.Nil(t, err)

	ticket, err := doc0.Share(iroh.ShareModeWrite)
	assert.Nil(t, err)

	// Subscribe to sync events
	hashCh := make(chan iroh.Hash)
	callback := &callback{
		hashCh: hashCh,
	}
	err = doc0.Subscribe(callback)
	assert.Nil(t, err)

	// Join the same doc from node_1
	doc1, err := node1.DocJoin(ticket)
	assert.Nil(t, err)

	// Create author on node_1
	author, err := node1.AuthorCreate()
	assert.Nil(t, err)
	_, err = doc1.SetBytes(author, []byte("hello"), []byte("world"))
	assert.Nil(t, err)

	// Wait for the content ready event
	hash := <-hashCh

	// Read the value using the found hash
	val, err := node1.BlobsReadToBytes(&hash)
	assert.Nil(t, err)

	// Assert
	expectedVal := []byte("world")
	assert.Equal(t, expectedVal, val)
}

type callback struct {
	hashCh chan iroh.Hash
}

func (c *callback) Event(event *iroh.LiveEvent) *iroh.IrohError {
	fmt.Println("event type", event.Type())
	if event.Type() == iroh.LiveEventTypeInsertLocal {
		fmt.Println("type insert local")
	}

	if event.Type() == iroh.LiveEventTypeInsertRemote {
		fmt.Println("type insert remote")
	}

	if event.Type() == iroh.LiveEventTypeNeighborUp {
		fmt.Println("type neighbor up")
	}

	if event.Type() == iroh.LiveEventTypeNeighborDown {
		fmt.Println("type neighbor down")
	}

	if event.Type() == iroh.LiveEventTypeSyncFinished {
		fmt.Println("type sync finished")
	}

	if event.Type() == iroh.LiveEventTypeContentReady {
		fmt.Println("type content ready found")
		c.hashCh <- *event.AsContentReady()
	}
	return nil
}
