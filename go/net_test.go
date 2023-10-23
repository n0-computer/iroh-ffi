package main

import (
	"testing"

	"github.com/n0-computer/iroh-ffi/iroh"
	"github.com/stretchr/testify/assert"
)

// TestIpv4Addr tests all IPv4Addr constructors and methods
func TestIpv4Addr(t *testing.T) {
	// create ipv4 addr from the constructor
	fromCons := iroh.NewIpv4Addr(10, 10, 10, 10)

	// create ipv4 addr from a string
	ipStr := "10.10.10.10"
	fromStr, err := iroh.Ipv4AddrFromString(ipStr)
	assert.Nil(t, err)

	// ensure the strings are what we expect,
	assert.Equal(t, fromCons.ToString(), ipStr)
	assert.Equal(t, fromStr.ToString(), ipStr)
	//
	// ensure octets are what we expect
	octets := []byte{10, 10, 10, 10}
	assert.Equal(t, fromCons.Octets(), octets)
	assert.Equal(t, fromStr.Octets(), octets)
}

// TestIpv6Addr tests all IPv6Addr constructors and methods
func TestIpv6Addr(t *testing.T) {
	// create ipv6 addr from the constructor
	fromCons := iroh.NewIpv6Addr(10000, 10000, 10000, 10000, 10000, 10000, 10000, 10000)
	//
	// create ipv6 addr from a string
	ipStr := "2710:2710:2710:2710:2710:2710:2710:2710"
	fromStr, err := iroh.Ipv6AddrFromString(ipStr)
	assert.Nil(t, err)
	//
	// ensure strings are what we expect,
	assert.Equal(t, fromCons.ToString(), ipStr)
	assert.Equal(t, fromStr.ToString(), ipStr)
	//
	// ensure segments are what we expect
	segments := []uint16{10000, 10000, 10000, 10000, 10000, 10000, 10000, 10000}
	assert.Equal(t, fromCons.Segments(), segments)
	assert.Equal(t, fromStr.Segments(), segments)
}

// TestSocketAddrV4 tests all SocketAddrV4 constructors and methods
func TestSocketAddrV4(t *testing.T) {
	// create an addr and a port
	ipv4, err := iroh.Ipv4AddrFromString("127.0.0.1")
	assert.Nil(t, err)
	var port uint16 = 3000
	socketAddrStr := "127.0.0.1:3000"
	ipStr := "127.0.0.1"

	// create a socket addrs
	fromCons := iroh.NewSocketAddrV4(ipv4, port)
	fromStr, err := iroh.SocketAddrV4FromString(socketAddrStr)
	assert.Nil(t, err)

	// test the ip addr and port are as expected
	assert.Equal(t, fromCons.Ip().ToString(), ipStr)
	assert.Equal(t, fromCons.Port(), port)

	assert.Equal(t, fromStr.Ip().ToString(), ipStr)
	assert.Equal(t, fromStr.Port(), port)

	// test that the ToString works as expected
	assert.Equal(t, fromCons.ToString(), socketAddrStr)
	assert.Equal(t, fromStr.ToString(), socketAddrStr)
}

// TestSocketAddrV6 tests all SocketAddrV6 constructors and methods
func TestSocketAddrV6(t *testing.T) {
	// create an addr and a port
	ipv6, err := iroh.Ipv6AddrFromString("::1")
	assert.Nil(t, err)
	var port uint16 = 3000
	socketAddrStr := "[::1]:3000"
	ipStr := "::1"

	// create a socket addrs
	fromCons := iroh.NewSocketAddrV6(ipv6, port)
	fromStr, err := iroh.SocketAddrV6FromString(socketAddrStr)
	assert.Nil(t, err)

	// test the ip addr and port are as expected
	assert.Equal(t, fromCons.Ip().ToString(), ipStr)
	assert.Equal(t, fromCons.Port(), port)

	assert.Equal(t, fromStr.Ip().ToString(), ipStr)
	assert.Equal(t, fromStr.Port(), port)

	// test that the ToString works as expected
	assert.Equal(t, fromCons.ToString(), socketAddrStr)
	assert.Equal(t, fromStr.ToString(), socketAddrStr)
}

// TestSocketAddr tests all SocketAddr constructors and methods
func TestSocketAddr(t *testing.T) {
	// create a ip addrs & port
	ipv4Ip, err := iroh.Ipv4AddrFromString("127.0.0.1")
	assert.Nil(t, err)
	ipv6Ip, err := iroh.Ipv6AddrFromString("::1")
	assert.Nil(t, err)
	var port uint16 = 3000

	// create socket addrs
	ipv4 := iroh.SocketAddrFromIpv4(ipv4Ip, port)
	ipv6 := iroh.SocketAddrFromIpv6(ipv6Ip, port)

	// ensure the types are as expected
	assert.Equal(t, ipv4.Type(), iroh.SocketAddrTypeV4)
	assert.Equal(t, ipv6.Type(), iroh.SocketAddrTypeV6)

	// ensure we can get the addrs out properly
	ipv4Addr, err := ipv4.AsIpv4()
	assert.Nil(t, err)

	ipv6Addr, err := ipv6.AsIpv6()
	assert.Nil(t, err)

	// ensure they are as expected
	assert.Equal(t, ipv4Addr.Ip().ToString(), ipv4Ip.ToString())
	assert.Equal(t, ipv6Addr.Ip().ToString(), ipv6Ip.ToString())
	assert.Equal(t, ipv4Addr.Port(), port)
	assert.Equal(t, ipv6Addr.Port(), port)
}
