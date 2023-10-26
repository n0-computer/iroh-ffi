// tests that correspond to the `src/key.rs` rust api
package main

import (
	"testing"

	"github.com/n0-computer/iroh-ffi/iroh"
	"github.com/stretchr/testify/assert"
)

// TestPublicKey tests all the constructors and methods of the PublicKey
func TestPublicKey(t *testing.T) {
	keyStr := "ki6htfv2252cj2lhq3hxu4qfcfjtpjnukzonevigudzjpmmruxva"
	fmtStr := "ki6htfv2252cj2lh"
	bytes := []byte("\x52\x3c\x79\x96\xba\xd7\x74\x24\xe9\x67\x86\xcf\x7a\x72\x05\x11\x53\x37\xa5\xb4\x56\x5c\xd2\x55\x06\xa0\xf2\x97\xb1\x91\xa5\xea")

	// create key from string
	key, err := iroh.PublicKeyFromString(keyStr)
	assert.Nil(t, err)

	// test methods are as expected
	assert.Equal(t, key.ToString(), keyStr)
	assert.Equal(t, key.ToBytes(), bytes)
	assert.Equal(t, key.FmtShort(), fmtStr)

	//create key from bytes
	key0, err := iroh.PublicKeyFromBytes(bytes)
	assert.Nil(t, err)

	// test methods are as expected
	assert.Equal(t, key0.ToString(), keyStr)
	assert.Equal(t, key0.ToBytes(), bytes)
	assert.Equal(t, key0.FmtShort(), fmtStr)

	// test eq method works
	assert.True(t, key.Equal(key0))
	assert.True(t, key0.Equal(key))

}
