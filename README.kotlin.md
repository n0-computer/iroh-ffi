# Iroh Kotlin

## Build & test

With [`cargo-make`](https://crates.io/crates/cargo-make) installed:

```sh
cargo make bindgen-kotlin   # generate the Kotlin binding + stage the cdylib
cargo make test-kotlin      # bindgen + ./gradlew test
```

## Requirements

- `java` + a JDK (21+)
- `gradle`
- [`kotlinc`](https://kotlinlang.org/docs/command-line.html)
- [`ktlint`](https://github.com/pinterest/ktlint)

## Android development

Install the NDK via Android Studio and point Cargo at it, e.g. with
`NDK=/path/to/ndk/<version>` and an android ABI target of `29`:

```toml
# .cargo/config.toml
[target.aarch64-linux-android]
ar = "<NDK>/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-ar"
linker = "<NDK>/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android29-clang"

[target.armv7-linux-androideabi]
ar = "<NDK>/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-ar"
linker = "<NDK>/toolchains/llvm/prebuilt/darwin-x86_64/bin/armv7a-linux-androideabi29-clang"

[target.i686-linux-android]
ar = "<NDK>/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-ar"
linker = "<NDK>/toolchains/llvm/prebuilt/darwin-x86_64/bin/i686-linux-android29-clang"

[target.x86_64-linux-android]
ar = "<NDK>/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-ar"
linker = "<NDK>/toolchains/llvm/prebuilt/darwin-x86_64/bin/x86_64-linux-android29-clang"
```

## References

- https://sal.dev/android/intro-rust-android-uniffi/
