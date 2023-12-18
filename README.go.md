# iroh-go

Instructions on how to use `iroh-ffi/iroh-go` in your project!

```
$ go get github.com/n0-computer/iroh-ffi/iroh-go/iroh
$ git submodule add https://github.com/n0-computer/iroh-ffi.git extern/iroh-ffi
$ make -f extern/iroh-ffi/InstallGo
$ go mod edit -replace=github.com/n0-computer/iroh-ffi/iroh-go=./extern/iroh-ffi/iroh-go
```
