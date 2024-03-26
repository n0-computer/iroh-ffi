# iroh-ffi: On Hiatus

After careful consideration, we've decided to pause development on the ffi project for the forseeable future. In order to truly meet our standards, we need to focus on tuning the our Rust API to deliver the best possible experience. When that API has settled into a `1.0`, we can re-visit opening up the API for other languages.

## Last Release: 0.12.0

The last release of any iroh FFI language will be version `0.12.0`.

Thank you anyone who has picked up iroh in our ffi languages. If you have any questions or just want to chat, feel free to reach out here or on our [discord](https://discord.gg/ktrtZvTk).

# iroh-ffi

> Foreign Function Interface (FFI) bindings for Iroh

This repo defines Python & Swift bindings for [iroh](https://github.com/n0-computer/iroh), which is written in Rust.

### Published Packages:

[Python](https://pypi.org/project/iroh/)
[Swift](https://github.com/n0-computer/iroh-ffi)



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
