# Iroh Kotlin

## Requirements

### CLI with `kotlinc`

- [`kotlinc`](https://kotlinlang.org/docs/command-line.html)
- [`ktlint`](https://github.com/pinterest/ktlint)
- [`jna`](https://github.com/java-native-access/jna)
- `kotlinx.coroutines`
  - download from https://repo1.maven.org/maven2/org/jetbrains/kotlinx/kotlinx-coroutines-core-jvm/1.6.4/kotlinx-coroutines-core-jvm-1.6.4.jar

### Android Development

- Install the NDK you want to use, using Android Studio.
- Configure rust to use it, eg.

with an NDK location of `NDK=/Users/dignifiedquire/Library/Android/sdk/ndk/25.2.9519653` and an android ABI target of `29`.
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
