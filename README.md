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

## Node.js

- Make sure to install a recent version of node.js and npm
- Run `npm i`
- Run `npm run build`

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
- maturin expects you to use `virtualenv` to manage your virtual environment

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

### Mac

#### Building
Ensure you have golang & rust installed.

Install `uniffi-bindgen-go`:

```
cargo install uniffi-bindgen-go --git https://github.com/NordSecurity/uniffi-bindgen-go --tag v0.2.0+v0.25.0
```

Build the bindings:
```
./build_go.sh
```

Or build in release mode:
```
./build_go.sh release
```

#### Running
Once you've built the bindings, run go normally:
```
cd iroh-go
go test ./...
```

### Linux
Ensure you have golang & rust installed.

Install `uniffi-bindgen-go`:

```
cargo install uniffi-bindgen-go --git https://github.com/NordSecurity/uniffi-bindgen-go --tag v0.2.0+v0.25.0
```

Build the bindings:
```
./build_go.sh
```

Or in release mode:
```
./build_go.sh release
```

#### Running

If you've used the build script to build the go bindings, it will also place the files in the correct locations.

Add the following to let go know where the dynamically linked files are located:

```
cd iroh-go
LD_LIBRARY_PATH="${LD_LIBRARY_PATH:-}:.iroh/ffi" \
CGO_LDFLAGS="-liroh -L .iroh/ffi" \
go <actual go command to build or run>
```

### Windows

### Building
Ensure you have golang & rust installed.

Install `uniffi-bindgen-go`:

```
cargo install uniffi-bindgen-go --git https://github.com/NordSecurity/uniffi-bindgen-go --tag v0.2.0+v0.25.0
```

Build the bindings:
```
cargo build
```

### Running
To make sure everything go needs to find is included the following is needed

```
LD_LIBRARY_PATH="${LD_LIBRARY_PATH:-}:<binaries path>" \
CGO_LDFLAGS="-liroh -L <binaries path>" \
go <actual go command to build or run>
```

where `<binaries path` needs to be replaced with the absolute path to where the rust build output is. Eg `/<path to repo>/iroh-ffi/target/debug` in debug mode.

#### Running

If you've used the build script to build the go bindings, it will also place the files in the correct locations.

Add the following to let go know where the dynamically linked files are located:

```
cd iroh-go
LD_LIBRARY_PATH="${LD_LIBRARY_PATH:-}:.iroh/ffi" \
CGO_LDFLAGS="-liroh -L .iroh/ffi" \
go <actual go command to build or run>
```



### Updating the bindings

# Developers
Check our our [DEVELOPERS.md](DEVELOPERS.md) for guides on how to translate from the iroh rust API to the iroh FFI API, as well as how to set up testing for golang and python.

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
