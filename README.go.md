# iroh-go

Instructions on how to use `iroh-ffi/iroh-go` in your project!

## mac & linux

```
$ go get github.com/n0-computer/iroh-ffi/iroh-go
$ git submodule add https://github.com/n0-computer/iroh-ffi.git extern/iroh-ffi
$ make -f extern/iroh-ffi/InstallGo
$ go mod edit -replace=github.com/n0-computer/iroh-ffi/iroh-go=./extern/iroh-ffi/iroh-go
```

## windows
Currently, to use iroh-go for windows, you need to build from source.

You will need rust and cargo installed on your machine.

```
$ go get github.com/n0-computer/iroh-ffi/iroh-go
$ git submodule add https://github.com/n0-computer/iroh-ffi.git extern/iroh-ffi
$ cd extern/iroh-ffi
$ cargo install uniffi-bindgen-go --git https://github.com/NordSecurity/uniffi-bindgen-go --tag v0.2.0+v0.25.0
$ cargo build --release
$ cd ../..
$ go mod edit -replace=github.com/n0-computer/iroh-ffi/iroh-go=./extern/iroh-ffi/iroh-go
$ export LD_LIBRARY_PATH="${LD_LIBRARY_PATH:-}:extern/iroh-ffi/iroh-go/iroh"
$ export CGO_LDFLAGS="-liroh -L extern/iroh-ffi/iroh-go/iroh"
```
Then you can properly run your go commands

