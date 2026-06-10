# Iroh Swift

## Build & test

With [`cargo-make`](https://crates.io/crates/cargo-make) installed:

```sh
cargo make test-swift        # build macOS slice + run swift test
cargo make swift-xcframework # full iOS + macOS xcframework (release)
```

## Xcode and iOS

- Run `cargo make swift-xcframework`.
- Add `IrohLib` as a local package dependency under `Frameworks, Libraries, and
  Embedded Content` in your project's `General` settings.
- Build. Confirm `IrohLib` is listed under `Frameworks, Libraries, and Embedded
  Content` (re-add with `+` if not).
- Add `SystemConfiguration` and `CoreWLAN` as Frameworks (iroh's netwatch needs
  them on Apple platforms).
- `import IrohLib` in Swift.
