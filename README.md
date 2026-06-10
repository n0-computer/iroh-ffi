# iroh-ffi

> Foreign Function Interface (FFI) bindings for Iroh

This repo defines Python, Swift, Kotlin and Node.js bindings for [iroh](https://github.com/n0-computer/iroh), which is written in Rust. The bindings mirror the stabilized iroh 1.0 surface (endpoints, connections, paths, tickets, relays, services); higher-level protocols not yet at 1.0 (`iroh-blobs`, `iroh-docs`, `iroh-gossip`) are out of scope.

### Published Packages

- [Python: PyPI](https://pypi.org/project/iroh/) — `iroh`
- [Swift: SwiftPM](https://swiftpackageindex.com/n0-computer/iroh-ffi)
- [Swift: Cocoapods](https://cocoapods.org/pods/IrohLib) — `IrohLib`
- [Kotlin / JVM: Maven Central](https://central.sonatype.com/artifact/computer.iroh/iroh) — `computer.iroh:iroh`
- [JavaScript: npm](https://www.npmjs.com/package/@number0/iroh) — `@number0/iroh`

## Documentation

Per-language API docs are at <https://n0-computer.github.io/iroh-ffi/>.

If you're blocked on something or need to draw attention to an issue, please reach out to the iroh [discord](https://discord.gg/B4pzE3usDC).


## Build & test

This repo uses [`cargo-make`](https://crates.io/crates/cargo-make) as its single build/test entry point:

```sh
cargo make test-all          # rust + python + js + kotlin + swift
cargo make test-rust         # just the Rust suite
cargo make bindgen-kotlin    # generate the Kotlin binding
cargo make swift-xcframework # full Apple xcframework (iOS + macOS)
```

See the per-language READMEs below and [DEVELOPERS.md](DEVELOPERS.md) for details. Release flow is documented in [RELEASING.md](RELEASING.md).

## Language-Specific READMEs

* [**Swift readme**](README.swift.md)
* [**Python readme**](README.python.md)
* [**Kotlin readme**](README.kotlin.md)
* [**Node.js readme**](iroh-js/README.md)

## Developers
Check our [DEVELOPERS.md](DEVELOPERS.md) for guides on how to translate from the iroh rust API to the iroh FFI API, as well as how to set up testing for Golang and Python.

## Community Bindings

The community has built additional language bindings that are open source and available for use. For the full list, see the official documentation: **[Using Iroh in Other Languages](https://docs.iroh.computer/deployment/other-languages)**

## Professional Bindings Support

The number0 engineering team can help you build and maintain production-grade language-specific bindings. [Contact us](https://iroh.computer/services/support) to discuss your requirements.

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
