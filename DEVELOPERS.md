# Developers

`iroh-ffi` exposes a minimal, 1.0-track slice of [`iroh`](https://github.com/n0-computer/iroh)
to Python, Swift, Kotlin, and JavaScript. Python/Swift/Kotlin use
[`uniffi-rs`](https://mozilla.github.io/uniffi-rs/); JavaScript uses
[`napi-rs`](https://napi.rs/).

## One tool for everything: `cargo make`

All build / test / bindgen / publish flows live in `Makefile.toml`. The same
commands run locally and in CI, so they never drift. Install once:

```sh
cargo install cargo-make
```

### Test

```sh
cargo make test-all        # rust + python + js + kotlin + swift
cargo make test-rust
cargo make test-python     # activate your virtualenv first (see below)
cargo make test-js
cargo make test-kotlin
cargo make test-swift      # macOS-arm64 slice (fast)
```

### Lint / format / docs

```sh
cargo make format          # rustfmt (workspace)
cargo make format-check
cargo make clippy
cargo make docs
cargo make ci-rust         # everything the Rust CI job runs
```

### Bindgen / packaging

```sh
cargo make bindgen-kotlin       # generate Kotlin binding + stage the cdylib
cargo make bindgen-swift-macos  # generate Swift binding + stage the macOS slice
cargo make swift-xcframework    # full 4-target Apple xcframework (release)
cargo make python-wheel         # release wheel into target/wheels
cargo make js-build-release     # stripped release napi addon
```

`cargo make --list-all-steps` shows every task with its description.

## Scope

The FFI mirrors the iroh 1.0 surface: `Endpoint`, `Connection`, streams,
datagrams, `EndpointId` / `EndpointAddr` / `EndpointTicket`, `SecretKey` /
`Signature`, custom relays (`RelayMap` / `RelayMode`), watchers, multipath
snapshots, and the `iroh-services` client. `docs`, `gossip`, and `blobs` are
intentionally **not** exposed (they are not 1.0 APIs).

## Translating the iroh API into bindings

General guidelines when surfacing an `iroh` API:

- `PathBuf` → `String`; `Bytes` / `[u8]` → `Vec<u8>`
- Anything streaming should read from / write to explicit buffers, or expose a
  `size` accessor so callers can decide how to handle the data.
- Methods returning a `Stream` (e.g. a `list`) should return a `Vec`. Add a
  comment warning that everything is loaded into memory.
- Methods that emit progress/events take a callback trait
  (`#[uniffi::export(with_foreign)]` on the uniffi side, `ThreadsafeFunction`
  on the napi side). See the watcher callbacks in `src/watch.rs`.
- Fallible methods return `IrohError` — follow the pattern in `src/error.rs`.
- Mirror upstream names. Where a name collides with a host-language builtin
  (e.g. Kotlin `AutoCloseable.close()`), rename **only** for that binding via
  `uniffi.toml`'s `[bindings.kotlin.rename]`, not globally.
- Every value type that has a sensible textual form implements `Display`
  (uniffi `#[uniffi::export(Display)]`) and a `to_string()`/`toString()` on the
  napi side, so it feels native in every language.

## Testing

When you add a piece of the API, add a test for it in **every** binding:
`src/*.rs` (`#[cfg(test)]`), `python/*_test.py`, `iroh-js/test/*.mjs`,
`kotlin/lib/src/test/kotlin/computer/iroh/*Test.kt`, and
`IrohLib/Tests/IrohLibTests/IrohLibTests.swift`. The end-to-end connectivity
tests use the loopback pattern from iroh's own suite (`Preset::N0` +
`RelayMode::disabled()`, dial via `endpoint.addr()`).

## Per-language notes

### Python

Install [`maturin`](https://www.maturin.rs/installation), create + activate a
virtualenv, then:

```sh
pip install pytest pytest-asyncio maturin uniffi-bindgen
cargo make test-python
```

`cargo make python-develop` builds + installs into the active venv;
`cargo make python-wheel` builds a release wheel. For a portable manylinux
wheel: `docker run --rm -v $(pwd):/mnt -w /mnt quay.io/pypa/manylinux_2_28_x86_64 /mnt/build_wheel.sh`.

### JavaScript

```sh
cargo make test-js   # yarn install + napi build --platform + node --test
```

Requires Node ≥ 20 and yarn.

### Kotlin

Requires `java`, `gradle`, and a JDK. `cargo make test-kotlin` generates the
binding and runs the Gradle suite. Android cross-builds are handled in the
release workflow.

### Swift

`cargo make test-swift` builds the macOS-arm64 slice and runs `swift test`.
`cargo make swift-xcframework` builds the full iOS + macOS xcframework. iroh's
netwatch needs `CoreWLAN` on macOS; it is declared in `IrohLib/Package.swift`.
