# Spike findings: `dylib` plugin linkage for iroh protocol FFI

**All four spikes PASS.** A separately-built, separately-published plugin library can share one
copy of `iroh` with the core library, and uniffi external types work across that boundary in
Python, Kotlin/JVM, and Swift. **Swift is proven on macOS only** — iOS still needs the
static→dynamic framework migration (finding 15).

Environment: macOS arm64 (Darwin 25.6.0), `rustc 1.97.1 (8bab26f4f 2026-07-14)`, JDK 17 launcher,
uniffi 0.31.2, iroh 1.0.x, iroh-ping 1.0.0.

Reproduce:

```sh
# Python
cargo build -p iroh-ffi-ping
bash spikes/stage_python.sh debug
cd target/spike-stage && python3 round_trip.py

# Kotlin (JDK 17 launcher — see the environment note at the end)
bash spikes/stage_kotlin.sh
cd kotlin && ./gradlew :lib:test :ping:test :ping:testExtracted

# Swift (macOS)
bash spikes/stage_swift.sh
IROH_FORCE_STAGING_RELAYS=1 target/swift-spike/.build/debug/SpikeMain
```

---

## Spike A — dylib linkage, one copy of `iroh`: **PASS**

`iroh-ffi` with `crate-type = ["staticlib", "dylib"]`, plugin `iroh-ffi-ping` with
`crate-type = ["cdylib"]` depending on it by path.

| artifact | debug | release |
|---|---:|---:|
| `libiroh_ffi.dylib` (core, contains iroh) | 163 MB | 42.8 MB |
| `libiroh_ffi_ping.dylib` (plugin) | **362 KB** | **308 KB** |

The plugin is ~0.7% of core's size — it carries **zero copies of `iroh`**. `otool -L`
confirms it links core dynamically:

```
libiroh_ffi_ping.dylib:
    @rpath/libiroh_ffi.dylib
    @rpath/libstd-4f24f0876fd27385.dylib
```

`nm -u` shows 127 undefined `iroh`-related symbols resolved from core, including the
load-bearing ones for external types:

```
Handle::into_arc::<iroh_ffi::net::EndpointAddr>
Handle::from_arc::<Arc<dyn iroh_ffi::endpoint::ProtocolHandler>>
```

Both sides use the **same monomorphization** of the uniffi handle lift, which is exactly
what makes passing an `Arc` handle between the two libraries sound. This is the flaw in the
old `iroh_gossip` 0.31.0 wheel (static core copy) that dynamic linkage fixes.

### Findings that change the plan

1. **`cargo` forbids `dylib` + `cdylib` on the same lib target**, so `cdylib` must be
   **replaced** by `dylib`, not supplemented (`staticlib` + `dylib` is allowed):
   ```
   error: library `iroh_ffi` cannot set the crate type of both `dylib` and `cdylib`
   ```
   A Rust `dylib` does export the `#[no_mangle]` uniffi C ABI (415 symbols in core, 63 in the
   plugin) and `ctypes` loads it fine.

   > ### ⚠ Correction
   > An earlier revision of this document claimed maturin silently produces an empty wheel
   > from a `dylib` crate. **That was wrong** — it was an artifact of the stale
   > `maturin 1.8.1` pinned in this repo's `.venv`. Retested with **maturin 1.14.1**: it
   > handles `["staticlib", "dylib"]` correctly, runs its uniffi step, and produces a
   > complete wheel (`iroh/iroh_ffi.py` 419,920 B + `iroh/libiroh_ffi.dylib` 70,412,592 B +
   > `__init__.py`, `py.typed`, `__init__.pyi`). No packaging change is needed.

   **The real hazard is artifact collision.** maturin's uniffi build overrides the crate type
   and writes a *statically-linked-std cdylib* to the same path cargo uses for the dylib
   (`target/debug/libiroh_ffi.dylib`). After a `maturin build`, plugin compilation fails
   until the dylib is rebuilt — see finding 2 for the error. CI and any dev loop that runs
   both must not share a target dir, or must rebuild between steps.

   **`pyproject.toml` must raise its maturin floor.** It currently allows
   `maturin>=1.2,<2.0`; the fixes below require **≥1.12.3**.

2. **`-C prefer-dynamic` IS required — for `std`, not for core.** Core linkage needs no flag:
   because `iroh-ffi` publishes no `rlib`, rustc must link it dynamically (so the concern from
   [rust-lang/rust#90539](https://github.com/rust-lang/rust/issues/90539) does not apply on
   1.97.1). But **`std` is a different matter.** Build the core dylib on its own and rustc
   links `std` *statically* into it; a plugin built against that dylib then cannot satisfy
   the single-copy rule and fails hard:
   ```
   error: cannot satisfy dependencies so `std` only shows up once
   error: cannot satisfy dependencies so `core` only shows up once
   ... (19 errors: alloc, compiler_builtins, libc, unwind, hashbrown, ...)
   ```
   This is the Reference's single-copy guarantee doing its job — and it is why the failure
   surfaced only after a `maturin build` clobbered the dylib (finding 1). Building the whole
   graph in one `cargo build` masks it, because cargo then makes `std` dynamic in both.

   **The published core artifact must therefore be built deliberately with
   `RUSTFLAGS="-C prefer-dynamic"`**, so third parties can link it later. Verified: core
   rebuilt with the flag gains `@rpath/libstd-...dylib`, and the plugin then builds and runs.
   This is a *contract*, not an implementation detail — a core dylib published without it is
   unusable by plugins.

3. **`libstd` must ship.** Both artifacts declare `@rpath/libstd-4f24f0876fd27385.dylib`.
   Without it, `dlopen` fails outright. It lives at
   `$(rustc --print sysroot)/lib/rustlib/<target>/lib/libstd-<hash>.dylib`, is 1.4 MB, is
   **per-target**, and the hash is **tied to the exact rustc version**. Every platform
   artifact must bundle the matching libstd, and the toolchain version becomes part of the
   published compatibility contract.

4. **`[profile.release] lto = true` is fine.** Release built clean in 1m35s. Note LTO cannot
   cross the dylib boundary, so plugin↔core calls are not inlined — a performance
   consideration, not a blocker.

---

## Spike B — two Python packages, one shared native library: **PASS**

```
target/spike-stage/
  iroh/       libiroh_ffi.dylib (162 MB)  libstd-...dylib (1.4 MB)  iroh_ffi.py (420 KB)
  iroh_ping/  libiroh_ffi_ping.dylib (378 KB)                       iroh_ffi_ping.py (41 KB)
```

Real round trip, two endpoints from **core's** library, protocol handler from the
**plugin's** library:

```
accepted connection from 44e59b3d...
  server bound: a00d6ad0... addrs=[192.168.0.171:59549, ...]
  client bound: 44e59b3d... addrs=[192.168.0.171:65500, ...]
  PING -> PONG round trip: 1 ms
  OK: Endpoint crossed the library boundary and the protocol ran
```

Generated plugin bindings resolve external types correctly — including the **callback
interface**, which was the riskiest case:

```python
import iroh.iroh_ffi

class Ping(PingProtocol):
    def handler(self) -> iroh.iroh_ffi.ProtocolHandler: ...
    async def ping(self, endpoint: iroh.iroh_ffi.Endpoint,
                         addr: iroh.iroh_ffi.EndpointAddr) -> int: ...
```

### Findings that change the plan

5. **The stock `uniffi-bindgen` CLI cannot do this.** It fails with:
   ```
   module lookup failed: "iroh_ffi::endpoint"
   ```
   Root cause: `BindgenLoader::load_metadata` reads `UNIFFI_META` symbols from **exactly one**
   library, and `module_path_map` is populated only from `Metadata::Namespace` items
   (`uniffi_bindgen/src/pipeline/initial/from_uniffi_meta.rs:79`). The plugin dylib holds 7
   META symbols; core holds 199. Core's namespace item is simply absent.

   **Fix, and it's cheap:** the public API supports merging. `spikes/bindgen/` is ~40 lines —
   `load_metadata` each library, `extend` the `MetadataGroupMap`, then
   `load_pipeline_initial_root` + `bindings::python::run_pipeline` with a crate filter.

   > **Superseded by finding 9.** Kotlin cannot be generated this way at all (its generator
   > internals are private), and the static-build-for-bindgen technique found while fixing
   > that works for both languages with the **stock CLI**. `spikes/bindgen/` is retained only
   > as the record of how this was diagnosed.

   **Upstream status: not fixed, and unsupported by design.** Checked uniffi 0.32.0
   (2026-06-30, the current release — this repo pins 0.31.1). No `--metadata-from` flag, no
   multi-library metadata loading; the 0.32.0 changelog has nothing on it. The maintainer
   position in [uniffi#2647](https://github.com/mozilla/uniffi-rs/issues/2647) (open, Sept
   2025) is explicit, stated three times by @mhammond:

   > "uniffi supports multiple crates fine, but **does not support multiple *libs*** - ie, you
   > need to build these crates into a single .a (for swift) or .so (for everywhere else)."
   >
   > "We certainly support many crates compiled into 1 library, but **do not support multiple
   > libraries**."

   So the spike works, but **outside the supported envelope**. There is no upstream guarantee,
   the `spikes/bindgen/` approach leans on API whose stability is only incidental, and a uniffi
   minor release could break it. Getting a `--metadata-from` flag upstreamed — or at least
   agreement that multi-library is a direction uniffi will accept — should be a precondition
   for betting the architecture on this, not an afterthought.

6. **The package name must match `[lib] name`.** uniffi records `crate_name` from
   `CARGO_CRATE_NAME`, which follows `[lib] name`, while the `cargo metadata` config layer
   keys `uniffi.toml` lookup off the *package* name. With `package = "iroh-ping-ffi-spike"`
   and `[lib] name = "iroh_ffi_ping"` the crate filter silently matched nothing (no `.py`
   written, no error) and `external_packages` would have been missed. Renaming the package to
   `iroh-ffi-ping` fixed both. This is a silent-failure footgun worth a check in the plugin
   template.

7. **Loader paths, macOS** (`spikes/stage_python.sh`). Cargo bakes an absolute build path as
   the install name, so packaging must rewrite it:
   ```sh
   install_name_tool -id     "@rpath/libiroh_ffi.dylib"      iroh/libiroh_ffi.dylib
   install_name_tool -change "<abs build path>" "@rpath/libiroh_ffi.dylib" \
                                                iroh_ping/libiroh_ffi_ping.dylib
   install_name_tool -add_rpath "@loader_path/../iroh"        iroh_ping/libiroh_ffi_ping.dylib
   codesign -f -s -                                          # required after install_name_tool
   ```
   Linux equivalent: `patchelf --set-rpath '$ORIGIN:$ORIGIN/../iroh'`. Windows
   (`os.add_dll_directory`) is **untested**.

8. **Version mismatch fails cleanly — better than expected.** Rebuilding core as 1.1.1 and
   loading a plugin built against 1.1.0 gives a load-time error, not a crash:
   ```
   OSError: dlopen(...libiroh_ffi_ping.dylib): Symbol not found:
     __RINvNtCs..._4core3ptr9drop_glueNtNtCs..._3noq11recv_stream14ReadToEndErrorECsluKi6jMjHJD_8iroh_ffi
   Expected in: ...libiroh_ffi.dylib
   ```
   Rust-mangled symbols embed the crate disambiguator (`CsluKi6jMjHJD_8iroh_ffi`), which folds
   in `-C metadata` — so a mismatched core cannot satisfy the plugin's imports. This is
   materially safer than Zenoh's situation, where stable C entry points with changed layouts
   SIGSEGV.

   **Not a guarantee, though.** It held because the plugin referenced a monomorphized generic
   naming core's disambiguator. A plugin whose referenced symbol set happened to be identical
   across two incompatible cores would load silently. So an explicit compatibility gate is
   still worth having — but as belt-and-braces, not as the only line of defence.

---

---

## Spike C — Kotlin/JVM, two Maven artifacts, one shared native library: **PASS**

Two separate Gradle modules → two separate JARs, each with its own native library:

| JAR | contains | size |
|---|---:|---:|
| `lib.jar` (`computer.iroh`) | `libiroh_ffi.dylib` 162 MB + `libstd-<hash>.dylib` 1.4 MB | 36 MB |
| `ping.jar` (`computer.iroh.ping`) | `libiroh_ffi_ping.dylib` **378 KB** | **174 KB** |

```
alpn = iroh/ping/0
server bound: 80b378ec... addrs=[192.168.0.171:64011, ...]
PING -> PONG round trip: 2 ms
```

28 tests green: 24 existing `:lib` (no regression), plus `:ping:test` (on-disk natives) and
`:ping:testExtracted` (the **published-consumer path**, where JNA unpacks each native out of
its JAR). Reproduce:

```sh
bash spikes/stage_kotlin.sh
cd kotlin && ./gradlew :lib:test :ping:test :ping:testExtracted
```

Kotlin shares types across packages properly, including core's `RustBuffer`:

```kotlin
package computer.iroh.ping
import computer.iroh.Endpoint
import computer.iroh.ProtocolHandler
import computer.iroh.RustBuffer as RustBufferEndpoint
```

### Findings that change the plan

9. **This kills the custom bindgen from finding 5 — use the stock CLI.** Kotlin's generator
   internals (`gen_kotlin::generate_bindings`, `Config`) are private, so the metadata-merging
   trick cannot be replicated for Kotlin at all. The way out is better than the workaround:
   **generate bindings from a *static* build and ship the *dynamic* one.**

   With `crate-type = ["staticlib", "dylib", "rlib"]` (allowed — only `dylib`+`cdylib` is
   rejected), a build *without* `-C prefer-dynamic` links core statically into the plugin, so
   the plugin library carries **both** crates' `UNIFFI_META` and the stock
   `uniffi-bindgen generate --library ... --crate iroh_ffi_ping` resolves external types with
   no custom tooling. Then rebuild with `-C prefer-dynamic` for the artifact you ship. The
   Python output is **byte-identical** to what `spikes/bindgen/` produced, so the metadata —
   and therefore the checksums — match the dynamic library. `spikes/stage_kotlin.sh` does both
   builds in order.

   **This removes the strategic risk in finding 5.** We no longer depend on
   incidentally-public bindgen API, only on documented CLI behaviour. `spikes/bindgen/` is
   kept for the record but is not needed.

10. **Never pass `--config` to bindgen in a multi-crate build.** `--config` is a global
    override applied to *every* crate, so core inherits the plugin's `package_name` and the
    plugin then imports its own package instead of core's — silently:
    ```kotlin
    import computer.iroh.ping.Endpoint   // wrong: core's Endpoint is computer.iroh.Endpoint
    ```
    Rely on per-crate `uniffi.toml` discovery via cargo metadata instead. Related: Kotlin's
    `external_packages` is keyed by **crate name** (`[lib] name`), not package name —
    `iroh_ffi`, not `iroh-ffi`.

11. **Load order is a hard requirement, and libstd is part of it.** The plugin declares
    `@rpath/libiroh_ffi.dylib` and `@rpath/libstd-<hash>.dylib`. JNA extracts each native out
    of its JAR to `~/Library/Caches/JNA/temp/jnaNNNN.tmp` — a mangled name in a directory
    where nothing else sits — so **no rpath can work**:
    ```
    UnsatisfiedLinkError: dlopen(.../jna17017633420966846699.tmp): Library not loaded:
      @rpath/libstd-4f24f0876fd27385.dylib
    ```
    dyld *will* satisfy an `@rpath/...` dependency from an already-loaded image with that
    install name, so the fix is purely ordering. Verified: with **zero rpaths** on the plugin,
    a 5-line shim makes the published path work —
    `kotlin/ping/src/main/kotlin/computer/iroh/ping/CoreNative.kt`:
    ```kotlin
    NativeLibrary.getInstance("std-4f24f0876fd27385")  // libstd first
    NativeLibrary.getInstance("iroh_ffi")              // then core
    ```
    **Every plugin package must ship this shim**, and it hard-codes the rustc-specific libstd
    hash — so the hash has to be generated, not written by hand. This makes finding 3's
    "toolchain version is part of the compatibility contract" concrete and visible in source.

12. **A uniffi error variant field named `message` generates Kotlin that will not compile.**
    ```kotlin
    class Failed(val `message`: kotlin.String) : PingException() {
        override val message get() = "message=${ `message` }"   // collides with itself
    }
    ```
    → `Conflicting declarations: val message` / `'message' hides member of supertype
    'Throwable'`. Worked around by naming the field `reason`. Worth reporting upstream; a
    plugin-authoring guide should call it out.

### Environment note, not a finding

`./gradlew` fails on this machine with a bare `26.0.2` error: Gradle 8.14.5 does not support
JDK 26 (Homebrew default here). Pre-existing and unrelated — `:lib:test` fails the same way on
a clean checkout. Run with `JAVA_HOME` pointed at JDK 17; the foojay resolver then provisions
JDK 21 for compilation.

---

## Spike D — Swift, two modules over two native libraries: **PASS on macOS, blocked on iOS packaging**

Two separate Swift modules, two separate C modules, two separately-built native libraries:

```
IrohLib  (module) + iroh_ffiFFI      (C module) -> libiroh_ffi.dylib      163 MB
IrohPing (module) + iroh_ffi_pingFFI (C module) -> libiroh_ffi_ping.dylib 362 KB
```

```
alpn = iroh/ping/0
server bound: c5082402... addrs=[192.168.0.171:53165, ...]
PING -> PONG round trip: 1 ms
OK: Endpoint crossed the Swift module + library boundary
```

Reproduce: `bash spikes/stage_swift.sh` then run the printed binary.

### Findings that change the plan

13. **The "single module" caveat is NOT binding — one injected import is enough.** uniffi
    documents: *"you must compile all generated `.swift` files together in a single module
    since the generated code expects that it can access external types without importing
    them."* Literally true — the plugin's generated Swift emits only
    `import Foundation` + `import iroh_ffi_pingFFI` and then references `Endpoint`,
    `EndpointAddr`, `ProtocolHandler`, `FfiConverterTypeEndpoint_lower`,
    `FfiConverterTypeProtocolHandler_lift` with no import and no declaration.

    But everything it needs is **`public`** in core's generated Swift
    (`public func FfiConverterTypeEndpoint_lower`, `open class Endpoint`,
    `public protocol EndpointProtocol`, …), so prepending a single line fixes it:
    ```swift
    import IrohLib
    ```
    Verified: `IrohPing` compiles as its own module and runs. This is the same class of
    one-line post-processing that `make_swift.sh` already does (it `sed`s `iroh_ffiFFI` → `Iroh`).

    Two C modules each declaring their own `RustBuffer` coexisted without complaint here.
    **Untested:** a plugin taking a *record* across the boundary, where `RustBuffer` itself
    would have to cross — that is the shape [uniffi#2802](https://github.com/mozilla/uniffi-rs/issues/2802)
    is about. This spike only passes objects.

14. **Swift has no `external_packages` config.** Its `Config`
    (`bindings/swift/gen_swift/mod.rs:167`) carries only `module_name`, `ffi_module_name`, and
    `rename` — there is no equivalent of the Python/Kotlin mechanism. The injected import is
    the whole answer, and it must be applied by the build, not by hand.

15. **The current static xcframework model cannot support plugins. Measured, not inferred.**
    Building the plugin as a `staticlib` produces **688 MB** against core's 686 MB — a Rust
    `staticlib` is self-contained by definition and re-embeds all of `iroh`, and
    `-C prefer-dynamic` does not change that. Both archives then export the **same 83
    `_ring_core_0_17_14__*` symbols**:
    ```
    _ring_core_0_17_14__aes_gcm_dec_kernel
    _ring_core_0_17_14__aes_gcm_enc_kernel
    _ring_core_0_17_14__aes_hw_ctr32_encrypt_blocks
    ...
    ```
    Linking both into one app is 83 duplicate-symbol errors, on top of shipping iroh twice.

    So Apple **must** move to dynamic frameworks (App-Store-legal when embedded and signed).
    `make_swift.sh` builds static `.a` slices and `[tasks.verify-swift-xcframework]` actively
    *fails* if a `.framework` directory appears — both need rewriting, and the
    `release_swift.yml` checksum flow along with them.

    **iOS itself remains untested.** This spike is macOS + dylibs, which proves the language
    and linkage story but not the framework packaging, embedding, or signing.

---

## Upstream tracking

### maturin — the `external_packages` wheel problem is **already fixed**

The packaging half of this was filed from this repo two years ago and has since been resolved:

| | |
|---|---|
| [maturin#2459](https://github.com/PyO3/maturin/issues/2459) | "Unable to use external dependency with uniffi" — filed 2025-01-29 with the exact `iroh-ffi` / `iroh-ffi-gossip` setup. **Closed 2026-02-17 as completed.** |
| [maturin#1904](https://github.com/PyO3/maturin/issues/1904) | "Uniffi multiple crates not supported" — missing `b.py` in the wheel. Closed 2026-02-16 as completed. |
| [maturin#3013](https://github.com/PyO3/maturin/pull/3013) | **The fix.** "exclude `external_packages` bindings from uniffi wheels" — *"When `uniffi.toml` defines `external_packages` in `[bindings.python]`, the generated Python bindings for those external crates are now excluded from the wheel and `__init__.py`. Previously they were included, causing import failures since the external crate's cdylib is not bundled in the wheel. Fixes #2459"* |
| [maturin#2839](https://github.com/PyO3/maturin/pull/2839) | Added the multi-crate uniffi test case. |

First release containing #3013: **maturin 1.12.3** (2026-02-19). That fix is precisely what a
plugin wheel needs, and it is why #2459's workaround — hand-commenting
`from .iroh_ffi import *` — is what the installed `iroh_gossip` 0.31.0 package still shows.
The only maintainer comment on #2459 before the fix is worth keeping in mind:

> @messense: "We don't have lots of usage of uniffi binding so support can be rough, pull
> requests are welcome if you'd like to improve it."

**Nothing needs filing on maturin.** The artifact-collision hazard in finding 1 is ours to
manage (separate target dirs), not a maturin bug.

### uniffi — the cross-library metadata problem is **open and out of scope upstream**

See finding 5 for the maintainer position on [uniffi#2647](https://github.com/mozilla/uniffi-rs/issues/2647).
Two more open issues bear on the deferred work:

- [uniffi#2802](https://github.com/mozilla/uniffi-rs/issues/2802) (open, Jan 2026, 16 comments)
  — "iOS linking conflicts when multiple UniFFI libraries are used": each library emits
  identical C types (`RustBuffer`), causing duplicate-symbol errors. Discussion favours a
  `c_type_prefix` / version-suffixed header, but @mhammond notes it **"wouldn't help for
  external types across frameworks, but I don't think this model would attempt to support
  that?"** — so the deferred Swift plan gets no help here. Unresolved as of April 2026.
- [uniffi#2763](https://github.com/mozilla/uniffi-rs/issues/2763), [#1896](https://github.com/mozilla/uniffi-rs/issues/1896)
  — adjacent "shared functionality / mixing generators" designs, no implementation.

**Two things worth filing/driving upstream**, both currently absent:
1. A `--metadata-from <lib>` flag (or blessing `BindgenLoader` metadata merging) so plugin
   authors don't each ship a custom bindgen.
2. A decision on whether multi-*library* is a direction uniffi will support at all. This is
   the single biggest strategic risk to the architecture.

## Verdict

The architecture works. Proceed, with these adjustments to the plan:

- Core ships as `["staticlib", "dylib"]` — **replacing** `cdylib` — built with
  `RUSTFLAGS="-C prefer-dynamic"` so `std` is dynamic and plugins can link it at all.
  maturin ≥1.12.3 handles this; raise the `pyproject.toml` floor from `>=1.2`. Keep maturin
  and plugin builds out of one another's target dir (finding 1). Re-verify the other existing
  consumers: JNA/Kotlin, Android `jniLibs`, and the `libiroh-<target>.tar.gz` C-lib artifacts.
- Every artifact bundles a per-target, rustc-pinned `libstd-<hash>.dylib`. The rustc version
  becomes part of the published compatibility contract.
- Bindings are generated from a **static** build and shipped against the **dynamic** one
  (finding 9), using the stock CLI — no custom bindgen, and `crate-type` gains `"rlib"`.
  Never pass `--config` (finding 10).
- Each plugin package ships a native-load-order shim with a generated libstd hash
  (finding 11).
- The compatibility gate stays in the plan, downgraded from "non-optional safety mechanism"
  to "clear diagnostics on top of an already-safe default failure".
- **Apple must move from the static xcframework to dynamic frameworks** (finding 15) — the
  static model is measurably impossible, not merely awkward. Swift also needs an `import`
  injected into the plugin's generated bindings (finding 13).
- **Residual strategic risk:** multi-library remains explicitly unsupported upstream
  (finding 5). Finding 9 removes our dependency on incidentally-public bindgen API, but not
  the risk that uniffi changes something that breaks the split. Worth an upstream
  conversation before betting on it.

## Not covered by these spikes

- **Windows** loader paths.
- **Kotlin/JNA**: whether JNA resolves the plugin's dependency on core, and Android `jniLibs`.
- **Swift**: still blocked on the static→dynamic framework migration plus uniffi's
  "all generated .swift in one module" constraint for external types.
- **JS/napi**: untouched; per-dylib class registry and `TypeId`-based `External` remain.
- **Protocol-to-protocol composition** (`iroh-docs` → `iroh-blobs` + `iroh-gossip`): plausible
  as ordinary Cargo deps between plugin crates, but the three-library case is unproven, and
  a plugin depending on another plugin must link *that* one as a dylib too.
- **A published-package test**: this used a staged directory, not real wheels via maturin.
