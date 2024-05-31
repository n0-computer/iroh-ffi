
# iroh-ffi

> Foreign Function Interface (FFI) bindings for Iroh

This repo defines Python & Swift bindings for [iroh](https://github.com/n0-computer/iroh), which is written in Rust.

### Published Packages:

[Python](https://pypi.org/project/iroh/)
[Swift](https://github.com/n0-computer/iroh-ffi)

## Repo Status: Tier 2

This is a "tier 2" repo, which means it's a repo we care about, but don't apply the same level of rigor as a "tier 1" repo. All work is done through pull requests, and must pass continuous integration, but the peer review process is much lighter, and our reaction times to issues will not be as fast as tier 1 repositories.

If you're blocked on something, or need to draw attention to an issue, please reach out the iroh [discord](https://discord.gg/B4pzE3usDC).

## Releases

We are _not_ commited to publishing releases for the FFI on a schedule or in tandem with the latest version of [iroh](https://github.com/n0-computer/iroh), but we try. There may be a gap between the latest version of `iroh` (or the latest verison of `iroh-ffi` that is released on github), and the matching versions of the python or swift ffi languages that are published.

If there currently is a gap, and you need a published python or swift package, please file an issue or reach out to us on our [discord](https://discord.gg/B4pzE3usDC).


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
- maturin expects you to use `virtualenv` to manage your virtual environment

### Building wheels

Invoking `maturin build` will build a wheel in `target/wheels`.  This
will likely only work on your specific platform. To build a portable
wheel for linux use:

```
docker run --rm -v $(pwd):/mnt -w /mnt quay.io/pypa/manylinux2014_x86_64 /mnt/build_wheel.sh
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
