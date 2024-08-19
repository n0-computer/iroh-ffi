# iroh-ffi

> Foreign Function Interface (FFI) bindings for Iroh

This repo defines Python, Swift, Kotlin and Node.js bindings for [iroh](https://github.com/n0-computer/iroh), which is written in Rust.

### Published Packages:

- [Python: pypi](https://pypi.org/project/iroh/)
- [Swift: Cocoapods](https://cocoapods.org/pods/IrohLib)
- [Swift: SwifPM](https://swiftpackageindex.com/n0-computer/iroh-ffi)
- [Rust: Crates](https://crates.io/crates/iroh)
- [JavaScript: `@number0/iroh`](https://www.npmjs.com/package/@number0/iroh)

## Repo Status: Tier 2

This is a "tier 2" repo, which means it's a repo we care about, but don't apply the same level of rigor as a "tier 1" repo. All work is done through pull requests and must pass continuous integration, but the peer review process is much lighter, and our reaction times to issues will not be as fast as tier 1 repositories.

If you're blocked on something or need to draw attention to an issue, please reach out to the iroh [discord](https://discord.gg/B4pzE3usDC).


## Library Compilation

Running `cargo build --release` will produce a dynamic library and a static library.

For builds targeting older versions of macOS, build with:  `MACOSX_DEPLOYMENT_TARGET=10.7 && cargo build --target x86_64-apple-darwin --release`.

## Language-Specific READMEs

* [**Swift readme**](README.swift.md)
* [**Python readme**](README.python.md)
* [**Kotlin readme**](README.kotlin.md)
* [**Node.js readme**](iroh-js/README.md)

## Developers
Check our [DEVELOPERS.md](DEVELOPERS.md) for guides on how to translate from the iroh rust API to the iroh FFI API, as well as how to set up testing for Golang and Python.

## License

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
