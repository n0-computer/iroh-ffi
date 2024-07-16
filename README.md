# iroh-ffi

> Foreign Function Interface (FFI) bindings for Iroh

This repo defines Python, Swift, and Kotlin bindings for [iroh](https://github.com/n0-computer/iroh), which is written in Rust.

### Published Packages:

[Python](https://pypi.org/project/iroh/)
[Swift](https://github.com/n0-computer/iroh-ffi)

## Repo Status: Tier 2

This is a "tier 2" repo, which means it's a repo we care about, but don't apply the same level of rigor as a "tier 1" repo. All work is done through pull requests and must pass continuous integration, but the peer review process is much lighter, and our reaction times to issues will not be as fast as tier 1 repositories.

If you're blocked on something or need to draw attention to an issue, please reach out to the iroh [discord](https://discord.gg/B4pzE3usDC).

## Releases

We are _not_ committed to publishing releases for the FFI on a schedule or in tandem with the latest version of [iroh](https://github.com/n0-computer/iroh), but we try. There may be a gap between the latest version of `iroh` (or the latest version of `iroh-ffi` that is released on GitHub), and the matching versions of the Python or Swift ffi languages that are published.

If there currently is a gap, and you need a published Python or Swift package, please file an issue or reach out to us on our [discord](https://discord.gg/B4pzE3usDC).


## Library Compilation

Running `cargo build --release` will produce a dynamic library and a static library.

For builds targeting older versions of macOS, build with:  `MACOSX_DEPLOYMENT_TARGET=10.7 && cargo build --target x86_64-apple-darwin --release`.

## Language Specifics

### Swift

[Notes here](https://github.com/n0-computer/iroh-ffi/blob/main/README.python.md)

### Python

[Notes here](https://github.com/n0-computer/iroh-ffi/blob/main/README.python.md)

### Kotlin

[Notes here](https://github.com/n0-computer/iroh-ffi/blob/main/README.kotlin.md)

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
