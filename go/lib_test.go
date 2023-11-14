package main

import (
	"testing"

	"github.com/n0-computer/iroh-ffi/iroh"
	"github.com/stretchr/testify/assert"
)

/// TestPathToKeyRoundtrip
func TestPathToKeyRoundtrip(t *testing.T) {
	path := "/foo/bar"
	key := []byte("/foo/bar\x00")

	// Test without prefix and root
	gotKey, err := iroh.PathToKey(path, nil, nil)
	assert.Nil(t, err)

	assert.Equal(t, key, gotKey)

	gotPath, err := iroh.KeyToPath(gotKey, nil, nil)
	assert.Nil(t, err)

	assert.Equal(t, path, gotPath)

	// Test with prefix
	prefix := "prefix:"
	key = []byte("prefix:/foo/bar\x00")

	gotKey, err = iroh.PathToKey(path, &prefix, nil)
	assert.Nil(t, err)

	assert.Equal(t, key, gotKey)

	gotPath, err = iroh.KeyToPath(gotKey, &prefix, nil)
	assert.Nil(t, err)

	assert.Equal(t, path, gotPath)

	// Test with root
	root := "/foo"
	key = []byte("prefix:bar\x00")

	gotKey, err = iroh.PathToKey(path, &prefix, &root)
	assert.Nil(t, err)

	assert.Equal(t, key, gotKey)

	gotPath, err = iroh.KeyToPath(gotKey, &prefix, &root)
	assert.Nil(t, err)

	assert.Equal(t, path, gotPath)
}
