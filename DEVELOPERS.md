# Developers

- This uses https://mozilla.github.io/uniffi-rs/ for building the interface

## translating the iroh API into iroh ffi bindings
Use these general guidelines when translating the rust API featured in the rust
`iroh` library with the API we detail in this crate:
- `PathBuf` -> `String`
- `Bytes`, `[u8]`, etc -> Vec<u8>
- Any methods that stream files or have `Reader` inputs or outputs should instead expect to read from or write to a file. Also, see if it's logical for any structs or enums that have a method to return a `Reader` to instead have `size` method, so that the user can investigate the size of the data before attempting to save it or load it in memory.
- Any methods that return a `Stream` of structs (such as a `list` method), should return a `Vec` instead. You should also add a comment that warns the user that this method will load all the entries into memory.
- Any methods that return progress or events should instead take a call back. Check out the `Doc::subscribe` method and the `SubscribeCallback` trait for how this should be implemented
- Most methods that return a `Result` should likely get their own unique `IrohError`s, follow the pattern laid out in `error.rs`.
- Except for unit enums, every struct and enum should be represented in the `udl` file as an interface. It should be expected that the foreign language use constructor methods in order to create structs, and use setters and getters to manipulate the struct, rather than having access to the internal fields themselves.
- Anything that can be represented as a string, should have a `to_string` and `from_string` method, eg `NamespaceId`, `DocTicket`
- Enums that have enum variants which contain data should look at the `SocketAddr` or `LiveEvent` enums for the expected translation.

## Testing

Please include tests when you add new pieces of the API to the ffi bindings

### Python

#### Requirements

- Install [`maturin`](https://www.maturin.rs/installation) for python development and packaging.
- Install `uniffi-bindgen` with `pip`
- `maturin develop` will build your package
- maturin expects you to use `virtualenv` to manage your virtual environment

#### pytest

We use [`pytest`](https://docs.pytest.org/en/7.1.x/contents.html) to test the python api.

Run the tests by using `python -m pytest` in order to correctly include all of the iroh bindings.


#### Building wheels

Invoking `maturin build` will build a wheel in `target/wheels`.  This
will likely only work on your specific platform. To build a portable
wheel for linux use:

```
docker run --rm -v $(pwd):/mnt -w /mnt quay.io/pypa/manylinux2014_x86_64 /mnt/build_wheel.sh
```

#### Translations

Uniffi translates the rust to python in a systematic way. The biggest discrepancy between the rust and python syntax are around how new objects are constructed

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

### Kotlin

- See `README.kotlin.md` for setup
- Run tests using `./test_kotlin.sh`
