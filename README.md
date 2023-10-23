# iroh-ffi 

> Foreign Function Interface (FFI) bindings for Iroh

This repo defines Python & Swift bindings for [iroh](https://github.com/n0-computer/iroh), which is written in Rust.

### Published Packages:

[Python](https://pypi.org/project/iroh/)
[Swift](https://github.com/n0-computer/iroh-ffi)

### Planned Support:
We're hoping to ship support for the following langauges in the future

- Kotlin



## Library Compilation

Running `cargo build --release` will produce a dynamic library and a static library.

For builds targeting older versions of MacOS, build with with:  `MACOSX_DEPLOYMENT_TARGET=10.7 && cargo build --target x86_64-apple-darwin --release`.

## Swift

### Xcode and IOS 

- Run `make_swift.sh`. 
- Add `IrohLib` as a local package dependency under `Frameworks, Libraries, and Embedded Content` in the `General` settings of your project. 
- Run `Build`
- Check that it is now listed under `Frameworks, Libraries, and Embedded Content`, if not click `+` again and add it from the list.
- Add `SystemConfiguration` as a Framework.
- Now you can just import the library in Swift with a standard import statement like `import IrohLib`.

## Python

- Install [`maturin`](https://www.maturin.rs/installation) for python development and packaging.
- Install `uniffi-bindgen` with `pip`
- `maturin develop` will build your package

### Building wheels

Invoking `maturin build` will build a wheel in `target/wheels`.  This
will likely only work on your specific platform. To build a portable
wheel for linux use:

```
docker run --rm -v $(pwd):/mnt -w /mnt quay.io/pypa/manylinux2014_x86_64 /mnt/build-wheel.sh
```

### Example

- Make sure the `iroh` is installed `pip install iroh`
- Run with `python3 main.py --help`

## Go

### Running 

To make sure everything go needs to find is included the following is needed

```
LD_LIBRARY_PATH="${LD_LIBRARY_PATH:-}:<binaries path>" \
CGO_LDFLAGS="-liroh -L <binaries path>" \
go <actual go command to build or run>
```

where `<binaries path` needs to be replaced with the absolute path to where the rust build output is. Eg `/<path to repo>/iroh-ffi/target/debug` in debug mode.


### Updating the bindings

Install `uniffi-bindgen-go`: 

```
cargo install uniffi-bindgen-go --git https://github.com/dignifiedquire/uniffi-bindgen-go --branch upgarde-uniffi-24
```

## Testing
Please include tests when you add new pieces of the API to the ffi bindings

### python

#### pytest
We use [`pytest`](https://docs.pytest.org/en/7.1.x/contents.html) to test the python api.

Ensure you have the correct virtualenv active, then run `pip install pytest`

Run the tests by using `python -m pytest` in order to correctly include all of the iroh bindings.

#### translations
Uniffi translates the rust to python in a systematic way. The biggest discrepency between the rust and python syntax are around how new objects are constructed

- constructor methods w/ `new` name:
    `Ipv4Addr::new(127, 0, 0, 1)` in rust would be `Ipv4Addr(127, 0, 0, 1)` in python
- constructor methods with any other name in rust:
    `SocketAddr::from_ipv4(..)` in rust would be `SocketAddr.from_ipv4(..)` in python
- method names will stay the same:
     `SocketAddr.as_ipv4` in rust will be called `SocketAddr.as_ipv4` in python
- unit enums will have the same names:
    `SocketAddrType::V4` in rust will be `SocketAddrType.V4` in python
- methods that return `Result` in rust will potentially throw exceptions on error in python

#### test file
Create a test file for each rust module that you create, and test all pieces of the API in that module in the python test file. The file should be named "[MODULENAME]\_test.py". For example, the `iroh::net` ffi bindings crate should have a corresponding "net\_test.py" file. 

### go 

#### go test 
Read the [Running](#running) section to ensure you include all the pieces necessary for running `go` commands (in this case, `go tests ./...`)

#### translations
Uniffi translates the rust to go in a systematic way. The biggest discrepency between the rust and go syntax are around how new objects are constructed. Here are the main differences

- constructor methods w/ the name `new` in rust:
    `Ipv4Addr::new(127, 0, 0, 1)` in rust would be `NewIpv4Addr(127, 0, 0, 1)` in go
- constructor methods that have any other name in rust:
    `SocketAddr::from_string(..)` in rust would be `SocketAddrFromString(..)` in go
- method names become PascalCase:
    `SocketAddr.as_ipv4` in rust will be called `SocketAddr.AsIpv4` in go 
- unit enums: 
    `SocketAddrType::V4` in rust will be `SocketAddrV4` in go 
- methods that return `Result` in rust:
    `Ipv4Addr::from_string(..)` returns `Result<String, IrohError>` in rust
    `Ipv4AddrFromString(..)` returns `String, IrohError` in go
    as an example:
    ```go
        ipv4Addr, err := Ipv4AddrFromString("127.0.0.1")
        if err != nil {
            // handle error here
        }
    ```

#### test file
Create a test file for each rust module that you create, and test all pieces of the API in that module in the go test file. The file should be named "[MODULENAME]\_test.go". For example, the `iroh::net` ffi bindings crate should have a corresponding "net\_test.go" file. 

## Development

- This uses https://mozilla.github.io/uniffi-rs/ for building the interface

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
