fn main() {
    // napi's `napi_*` symbols are provided by the Node host at runtime, not by any library.
    // `napi_build::setup()` emits `cargo:rustc-cdylib-link-arg`, which applies only to
    // cdylibs — a Rust `dylib` needs the same flag via `rustc-link-arg`.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup");
    }
}
