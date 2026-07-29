---
title: Shipping iroh protocols as language bindings
state: prediscussion
authors: Frando
---

# RFD: Shipping iroh protocols as language bindings

## Introduction

`iroh-ffi` today exposes exactly one thing: the endpoint layer. If you write
Python, Kotlin, Swift or JavaScript, you get endpoints, connections, streams and
an accept loop — and then you are on your own. Every protocol built on top of
iroh (`iroh-blobs`, `iroh-gossip`, `iroh-ping`, and anything a user writes) is a
Rust crate, reachable only from Rust.

That is the gap this RFD is about. We want `iroh-blobs` and `iroh-gossip` to be
usable from every language we bind, with the protocol wrapper living in the
protocol's own repository rather than being vendored into `iroh-ffi`.

The obvious approach — give each protocol crate its own uniffi/napi wrapper and
ship it as its own native library — has an obvious problem: a Rust protocol
crate is written against `iroh::Endpoint`, a Rust type with no stable ABI. Build
it as a standalone native library and you get a second, complete copy of iroh
inside it. Two protocols means three copies of iroh in the process, with three
sets of statics, three tokio runtimes' worth of machinery, and endpoints from one
copy silently invalid in another.

**The question this RFD answers: can a protocol crate ship its own FFI wrapper
without re-including iroh, on every platform we support?**

The answer is yes, two different ways, and both are prototyped and measured
here. The interesting result is that the approach which *looks* like it solves
the problem — dynamic linking — costs 20 MB more on disk than the approach that
sidesteps it.

## Background

### What we ship today

One crate, one native library per platform, four bindings:

- `iroh-ffi` → `libiroh_ffi.{so,dylib,dll}` (uniffi) → Python wheel, Kotlin AAR,
  Swift xcframework
- `iroh-js` → `iroh.node` (napi-rs) → npm package

Release config is `crate-type = ["staticlib", "cdylib"]` with `lto = true`. The
Apple artifact is a **static** xcframework (`xcodebuild -create-xcframework
-library`, `.a` + headers, no `.framework` bundles — see `make_swift.sh`).

### How protocols get registered

There is no exported `Router` object. `Endpoint::bind` takes
`EndpointOptions.protocols: HashMap<Vec<u8>, Arc<dyn ProtocolCreator>>` and
spawns the router internally (`src/endpoint.rs:318`). `ProtocolCreator` and
`ProtocolHandler` are `#[uniffi::export(with_foreign)]` traits, so both foreign
languages *and* other Rust crates can implement them.

This turns out to be exactly the right shape for what we want. A protocol
wrapper does not need to own an endpoint or spawn a router — it only needs to
export something that is a `ProtocolCreator`. The consumer composes the two, in
their own language.

### Requirements

These are the constraints a solution has to meet, in priority order.

1. **A protocol library must not contain a second copy of iroh.** This is the
   stated problem.
2. **Language consumers install nothing beyond their normal package manager.**
   `pip install`, `npm install`, Gradle, SwiftPM. No Rust toolchain, no rustup,
   no environment variables, no manual library placement. If a consumer ever has
   to run `rustup`, the design has failed — that cost is larger than any benefit
   on offer here.
3. **Every platform we ship today keeps working:** Linux gnu + musl (x86_64,
   aarch64, armv7), Android (4 ABIs), Apple (macOS, iOS, iOS-sim), Windows MSVC
   (x86_64, aarch64).
4. **Failures are loud.** A version mismatch between the core and a protocol
   library must fail visibly, not corrupt memory.

Requirement 2 is the one to keep hold of while reading the rest of this
document. Section [Dynamic std is our problem](#dynamic-std-is-our-problem-not-the-consumers)
addresses it head on, because it is the requirement Design B most obviously
threatens.

## Proposal

Two designs are viable. They differ only in packaging — **the Rust source of a
protocol wrapper is identical under both.**

### Design A — one native library, many crates

Keep shipping one native library. Add protocol crates to it as additional uniffi
namespaces.

```rust
// the combining crate — contains no API of its own
iroh_ffi::uniffi_reexport_scaffolding!();
iroh_ping_ffi::uniffi_reexport_scaffolding!();
iroh_blobs_ffi::uniffi_reexport_scaffolding!();
```

Each protocol crate calls `uniffi::setup_scaffolding!("iroh_ping")` and gets its
own namespace. No cross-crate type declaration is needed: `#[derive(uniffi::Object)]`
emits `impl<UT> FfiConverter<UT>`, generic over the crate tag, so a protocol
crate can name `iroh_ffi::Endpoint` directly.

(`uniffi::use_remote_type!` is *not* the tool for this — that is for wrapping
types from non-uniffi crates, and using it here produces a coherence error,
`conflicting implementations of trait uniffi::TypeId`.)

#### What Design A means for downstream users

This is the part that matters, because "one library" sounds like "one module"
and it is not.

**Python.** Today: `import iroh`. After: `import iroh` and `import iroh_ping` —
separate modules, separate namespaces, generated as separate `.py` files.
Whether they arrive as one wheel with two modules or two wheels sharing a
library is our choice; two wheels is fine as long as they are versioned
together. The generated code does the right thing across the boundary:

```python
# iroh_ping.py, generated
from . import iroh
def ping(self, endpoint: iroh.Endpoint) -> int:
    iroh._UniffiFfiConverterTypeEndpoint.check_lower(endpoint)
```

**Kotlin.** Separate packages, clean imports:
`import uniffi.iroh.Endpoint` inside `package uniffi.iroh_ping`. One AAR.

**JavaScript.** `require('@number0/iroh')` and `require('@number0/iroh-ping')`,
both resolving to the same `.node` file.

**Swift.** Here Design A costs something real. uniffi's Swift generator emits no
cross-namespace `import` — `iroh_ping.swift` calls
`FfiConverterTypeEndpoint_lower(...)`, a `public func` defined in `iroh.swift`,
with no import statement. So **all generated `.swift` must compile into one
Swift module**. Swift users get `import Iroh` with the ping types inside it, not
`import IrohPing`. Kotlin and Python have no such limitation. Fixing this means
patching uniffi upstream or hand-writing a shim module. *This limitation applies
to Design B equally* — it is a property of the Swift generator, not of the
linking strategy.

**What downstream users cannot do:** add a protocol we did not build. The set is
fixed when we build the library. A user with their own Rust protocol crate has
no way to get it into the shipped artifact.

**Usage is otherwise unchanged.** The consumer composes protocol and endpoint
themselves:

```python
ping = iroh_ping.Ping()
ep = await iroh.Endpoint.bind(iroh.EndpointOptions(protocols={ping.alpn(): ping}))
rtt = await ping.ping(other_ep, peer_id, addrs)
```

### Design B — split libraries, dynamically linked

Ship `iroh-ffi` as a Rust `dylib` and each protocol wrapper as a `cdylib` that
links it dynamically, everything built with `-C prefer-dynamic`.

```
libiroh_ffi.so          31 MB   the shared core
libiroh_ping_ffi.so    280 KB   ← no copy of iroh
libstd-<hash>.so       5.4 MB   shipped by us, see below
```

`-C prefer-dynamic` is mandatory. Without it rustc refuses outright:

```
error: cannot satisfy dependencies so `memchr` only shows up once
```

That error is rustc enforcing "each crate appears exactly once in the final
linkage" — which is precisely the guarantee we want. If it links, there is one
copy of iroh.

Downstream ergonomics are identical to Design A, including the Swift
single-module limitation. The difference is purely that protocol libraries are
separately shipped artifacts, which means **a third party could publish one.**

### Dynamic std is our problem, not the consumer's

Design B needs a dynamically linked `std`, which means shipping
`libstd-<hash>.so` alongside our libraries. This must be invisible: it is one
more binary inside the wheel / npm package / AAR / xcframework, exactly like the
main library. **A consumer must never install a Rust toolchain, run rustup, set
`LD_LIBRARY_PATH`, or place a file by hand.** If any of that were required,
Design B would not be worth considering.

That is a requirement, so it was tested rather than assumed.
`tests/test_p3x.py` builds the realistic layout — two separate packages, in
separate directories, linked only by a relative rpath baked in at build time:

```
out/p3x/iroh_core/   p3_iroh_core.py, libp3_iroh_core.so, libstd-<hash>.so
out/p3x/iroh_ping/   p3_iroh_ping_ffi.py, libp3_iroh_ping_ffi.so
```

```sh
RUSTFLAGS="-C prefer-dynamic -C link-arg=-Wl,-rpath,\$ORIGIN/../iroh_core" cargo build ...
```

Run with `LD_LIBRARY_PATH` explicitly unset (the test asserts it is unset), it
pings successfully. `readelf -d` confirms `RUNPATH: [$ORIGIN/../iroh_core]`. So
on ELF platforms the requirement is met with a standard mechanism and no
consumer-visible cost.

The same mechanism exists everywhere we ship, with different spellings, and each
is packaging work we would own:

| platform | mechanism | status |
|---|---|---|
| Linux, Android | `$ORIGIN` rpath | **verified** (`test_p3x.py`) |
| Apple | `@rpath` / `@loader_path` install names | standard, not tested here |
| Windows | no rpath; `LOAD_WITH_ALTERED_SEARCH_PATH` (Node does this) or `os.add_dll_directory` (Python ≥3.8) | per-language shim needed |

Windows is the one that needs real work rather than a linker flag, because it
has no rpath equivalent and the fix differs per language runtime.

## Evidence

Four prototypes, all runnable. Measurements are from this machine: rustc 1.95.0,
x86_64-unknown-linux-gnu, uniffi 0.31.2, napi-rs 3.12.

```
p1-combine/   uniffi, two crates → ONE cdylib            (Design A, minimal)
p2-dylib/     uniffi, two crates → TWO .so, shared       (Design B, minimal)
p3-real/      real iroh + real iroh-ping, BOTH designs
p4-napi/      napi-rs, two crates → TWO .node, shared    (Design B, for JS)
tests/        runnable end-to-end tests
out/          generated bindings + staged libraries (gitignored)
```

`p3-real/iroh-ping-ffi` has a path dependency on `../../../../iroh-ping`, i.e.
it assumes `iroh-ping` is checked out as a sibling of `iroh-ffi`.

### The protocol wrapper we would actually write

`p3-real/iroh-ping-ffi/src/lib.rs` is the shape being proposed. It does not
spawn a router and does not own an endpoint:

```rust
#[derive(Debug, uniffi::Object)]
pub struct Ping(iroh_ping::Ping);

#[uniffi::export]
impl Ping {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> { ... }
    pub fn alpn(&self) -> Vec<u8> { iroh_ping::ALPN.to_vec() }
    #[uniffi::method(async_runtime = "tokio")]
    pub async fn ping(&self, endpoint: Arc<Endpoint>, ...) -> Result<u64, CoreError> {
        self.0.ping(endpoint.raw(), addr).await  // <- needs `raw()` to be pub
    }
}

/// This is what lets it go into `EndpointOptions.protocols`.
#[uniffi::export]
impl ProtocolCreator for Ping {
    fn create(&self, _endpoint: Arc<Endpoint>) -> Arc<dyn ProtocolHandler> { ... }
}
```

uniffi generates `class Ping(PingProtocol, p3_iroh_core.ProtocolCreator)` in
Python — so the consumer drops it straight into `EndpointOptions`, exactly like
a protocol they implemented in Python themselves.

### It works, over a real connection

`tests/test_p3.py` binds two endpoints, registers ping on one, and pings it:

```
server id       : f07a9c715713255ffede0eb7b58a2b6f2b68a285baec8089732f4cbedc145668
core static addr: 0x7f9eb3da4be6
server addrs    : ['0.0.0.0:38485', '[::]:45452']
ping rtt        : 12729 us
pings sent      : 1
pings received  : 1
OK: real iroh ping across two separately-shipped .so files
```

### The same consumer code runs under both designs

`tests/test_p3a.py` is the same program against **one combined library** instead
of two. The only difference is which package the modules come from — decided by
`cdylib_name` at bindgen time, not by anything the consumer writes.

This is the evidence for the recommendation below: **choosing A does not close
the door on B.** The protocol wrapper crates written today are reusable
unchanged.

### Sizes

Release, stripped, x86_64-linux, same code throughout (iroh + iroh-ping + both
FFI wrappers):

| build | size |
|---|---:|
| Design A — one cdylib, `lto = true` | **10.6 MB** |
| Design A — one cdylib, `lto = false` | 15.4 MB |
| Design B — core `dylib` 30.8 MB + ping `cdylib` 0.27 MB | **31.0 MB** |
| *(for reference)* a naive standalone ping library with its own copy of iroh | 15.4 MB |

Design B does exactly what it promised: the protocol library drops from 15.4 MB
to 280 KB. And the total still nearly triples. Two causes:

- **~5 MB from losing LTO.** `lto = true` is incompatible with
  `prefer-dynamic` (`error: cannot prefer dynamic linking when performing LTO`).
  `lto = "thin"` fails identically. Worse, the error only fires for `bin`
  targets — for a `dylib` lib target cargo just **omits `-C lto` from the rustc
  invocation with no diagnostic at all**. Confirmed with `cargo build -v`.
- **~15 MB from the dylib itself.** A Rust `dylib` must export every public
  symbol, since it cannot know which ones a downstream Rust crate will call. So
  the linker cannot dead-strip, and almost nothing in iroh's dependency tree
  gets dropped.

The second is the larger effect and is not fixable on stable: you cannot narrow
a Rust dylib's export set without knowing the downstream call graph, and
`-Zdylib-lto` is nightly-only.

So Design B only breaks even on total bytes at roughly two separately-shipped
protocols — and Design A never pays the cost at all, because it shares iroh
statically inside one LTO'd library.

### Platform support

`crate-type = ["dylib"]` + `-C prefer-dynamic` requires rustup to ship a dynamic
`std` for the target. Checked directly against the installed toolchain rather
than assumed:

| target | dynamic std | notes |
|---|---|---|
| `x86_64-unknown-linux-gnu` | yes | built and ran end to end |
| `aarch64-unknown-linux-gnu` | yes | |
| `*-unknown-linux-musl` | yes | needs `-C target-feature=-crt-static` |
| `aarch64-linux-android` | yes | **cross-built and verified** |
| `armv7-linux-androideabi`, `i686-linux-android`, `x86_64-linux-android` | yes | |
| `aarch64-apple-ios` | yes | rustc emits `-dynamiclib`; link needs the SDK |
| `x86_64-apple-darwin` | yes | |
| `x86_64-pc-windows-msvc` | yes (`std-*.dll` + import lib) | rustc reached the link step; failed only on missing Windows SDK libs |

I expected Windows and iOS to be blockers. Neither is. The Android cross-build
produced the right thing:

```
$ llvm-readelf -d libp2_ext_lib.so
  (NEEDED)  libp2_core_lib.so
  (NEEDED)  libstd-04b6e765d4ef8976.so
```

What each platform needs is packaging work, not a workaround:

- **Android** — ship `libstd-<hash>.so` in `lib/<abi>/`. The linker resolves
  `DT_NEEDED` from the APK's native library directory.
- **Apple** — this is the largest single chunk of work. Design B forces a switch
  from the current static xcframework to dynamic `.framework` bundles, plus
  `@rpath` install names, plus code signing, plus `libstd-*.dylib` in every
  slice.
- **Windows** — no rpath; see the table in
  [Dynamic std is our problem](#dynamic-std-is-our-problem-not-the-consumers).

### JavaScript / napi-rs

`p4-napi/` mirrors the uniffi split: the core as a Rust `dylib` (which still
exports `napi_register_module_v1`, so Node loads it directly as an addon) and
the protocol as a `cdylib` linking it dynamically. `tests/test_p4.js` passes a
`core.Endpoint` into `ext.Ping.ping()`. The type checks hold across the
boundary:

```
ping(plain object) threw: Failed to recover `Endpoint` type from napi value
ping(wrong class)  threw: Value is not an instance of class `Endpoint`
```

Two napi-specific warts to plan around:

1. **The protocol addon re-exports the core's classes.** `Object.keys(ext)` is
   `['Ping', 'Endpoint']`, because napi-rs's `ctor`-based registry lives in the
   shared `napi` crate and every addon registers everything in it. The generated
   `index.js` / `index.d.ts` need filtering.
2. **`core.Endpoint !== ext.Endpoint`**, so `x instanceof ping.Endpoint` is
   `false` for a core-made endpoint. napi's internal unwrap check is `TypeId`-
   based and does the right thing, but user-facing `instanceof` and the
   TypeScript nominal types will lie unless the protocol package re-exports the
   core's constructor.

### Failure modes

Rust symbol names embed the crate SVH:

```
U _ZN11p2_core_lib8Endpoint5inner17hcf678c8a6f0905a1E
```

Rebuilding the core with anything different — different rustc, different source,
different feature resolution — changes those hashes. Tested by rebuilding with a
changed `-C metadata`:

```
load failed: OSError: libp2_ext_lib.so: undefined symbol:
  _ZN11uniffi_core3ffi6handle6Handle8into_arc17h4d3d245f1e3aad46E
```

This satisfies requirement 4 — loud, at load time, not silent memory corruption.
The cost is that the contract is exact: every protocol artifact must be rebuilt
whenever the core is, and **the rustc version is part of the public contract.**
In practice that means one workspace, one CI job, lockstep releases, and a
pinned `rust-toolchain.toml`.

### Tooling gap: bindgen needs one library

`uniffi-bindgen` takes one `<SOURCE>` and does not follow `DT_NEEDED`:

```
$ uniffi-bindgen generate --library libiroh_ping_ffi.so --language python
module lookup failed: "iroh_ffi"
```

Workaround, used by `p2-dylib/bindgen-lib` and `p3-real/bindgen-lib`: a
build-time-only cdylib that statically links every component, built *without*
`prefer-dynamic`, purely so bindgen has one file to read. Never shipped. Each
crate's `uniffi.toml` sets its own `cdylib_name` so the generated modules load
the right shipped library. It works, but it means building everything twice.

Design A does not have this problem — the shipped library *is* the one bindgen
reads.

## Determinations

**Adopt Design A now.**

It meets requirement 1 already: because iroh is shared statically inside one
LTO'd library, adding a protocol costs roughly what the protocol crate itself
costs. It meets requirements 2, 3 and 4 trivially, since nothing about
packaging changes. It is 20 MB smaller than the alternative. And it needs no
changes to the xcframework, the wheel layout, the npm package, or CI beyond
teaching them about N namespaces instead of one.

Work required:

- Promote `Endpoint::raw()` — and the equivalents on `Connection`, `EndpointId`,
  `RelayUrl` — from `pub(crate)` to `pub`. This makes the Rust-level inner types
  part of `iroh-ffi`'s public API, a new semver surface to maintain. Both
  designs need this.
- Write `iroh-ping-ffi`, `iroh-blobs-ffi`, `iroh-gossip-ffi` in the shape of
  `p3-real/iroh-ping-ffi`.
- Add the combining crate with `uniffi_reexport_scaffolding!()` calls.
- Teach the bindgen and packaging scripts about multiple namespaces.
- Decide the Swift story: accept one `import Iroh` module, or invest in the
  uniffi generator.

**Revisit Design B only when a third party shipping a protocol binding becomes a
real requirement.** It demonstrably works — every platform, real iroh, real
connection — so the door is open, and the wrapper crates written for A carry
over unchanged. But it buys modularity of *release*, and the price is +20 MB, no
LTO, a dynamic-xcframework rewrite, a doubled build, an exact rustc pin, and a
Windows DLL-search shim per language.

If that requirement does land, the alternative worth costing first is keeping
Design A and letting third parties get their protocol into the combined build —
a manifest of protocol crates plus a build service — rather than reaching for a
linking trick.

## Open questions

- **Swift.** Is one `import Iroh` module acceptable indefinitely, or do we
  invest in cross-namespace imports in uniffi's Swift generator? This is the
  only downstream-visible limitation of Design A, and it is not actually caused
  by Design A.
- **Wheel / npm layout.** One package with N modules, or N packages? Design A
  permits either. N packages is nicer for discovery and versions them
  independently in appearance only — they would still be released in lockstep.
- **`raw()` as public API.** Exposing the inner `iroh::Endpoint` makes iroh's own
  types part of `iroh-ffi`'s semver surface. Is that acceptable, or should there
  be a narrower `#[doc(hidden)]` protocol-author API?
- **Blobs specifically.** Ping and gossip are stream-shaped and fit this model
  cleanly. `iroh-blobs` has a much richer API and throughput requirements —
  worth confirming its wrapper fits the `ProtocolCreator` shape before
  committing.

## Alternatives considered

**Consumer-side assembly via macros.** Downstream users combine FFI-ready crates
and build their own native library. Rejected: it requires every consumer to have
a Rust toolchain, which violates requirement 2 outright. It also does not remove
the version coupling — it moves it onto the consumer's machine.

**A stable C-ABI plugin boundary.** Protocol crates written against a C API of
iroh rather than against `iroh` itself. This is the only design that would give
genuinely independent versioning across rustc versions. It is also by far the
most work: designing and maintaining a C API for endpoints, connections and
streams, and rewriting each protocol against a Rust wrapper over it instead of
against `iroh::Endpoint`. Plausible for ping and gossip; questionable for blobs
on both API-richness and throughput grounds. Not prototyped. Worth costing out
only if Design B turns out to be insufficient.

## Reproducing

Assumes `cd research-link-ffi`. `$STD` is the dynamic std for your host:

```sh
STD=$(find "$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/host: //p')/lib" -name 'libstd-*.so')
```

```sh
# --- Design A: one cdylib, two namespaces (minimal) ---
cargo build -p combined
cargo run -p combined --bin uniffi-bindgen -- generate \
    --library target/debug/libcombined.so --language python --out-dir out/python --no-format
cp target/debug/libcombined.so out/python/ && touch out/python/__init__.py
python3 tests/test_p1.py

# --- Design B: two .so files (minimal, no iroh — fast to iterate on) ---
cargo build -p p2-bindgen-lib && mkdir -p out/p2/python
cargo run -q -p p2-core-lib --bin p2-uniffi-bindgen -- generate \
    --library target/debug/libp2_bindgen_lib.so --language python \
    --out-dir out/p2/python --no-format
RUSTFLAGS="-C prefer-dynamic -C link-arg=-Wl,-rpath,\$ORIGIN" CARGO_TARGET_DIR=target-dyn \
    cargo build -p p2-core-lib -p p2-ext-lib
cp target-dyn/debug/libp2_{core,ext}_lib.so "$STD" out/p2/python/
touch out/p2/__init__.py out/p2/python/__init__.py
python3 tests/test_p2.py

# --- real iroh, shared build steps ---
cargo build -p p3-bindgen-lib                        # static; for bindgen metadata only

# --- Design A with real iroh ---
mkdir -p out/p3a/python
printf '[bindings.python]\ncdylib_name = "p3_bindgen_lib"\n' > /tmp/designa.toml
cargo run -q -p p3-iroh-core --bin p3-uniffi-bindgen -- generate \
    --library target/debug/libp3_bindgen_lib.so --language python \
    --out-dir out/p3a/python --no-format -c /tmp/designa.toml
cp target/debug/libp3_bindgen_lib.so out/p3a/python/
touch out/p3a/__init__.py out/p3a/python/__init__.py
python3 tests/test_p3a.py

# --- Design B with real iroh: two .so files ---
mkdir -p out/p3/python
cargo run -q -p p3-iroh-core --bin p3-uniffi-bindgen -- generate \
    --library target/debug/libp3_bindgen_lib.so --language python \
    --out-dir out/p3/python --no-format
RUSTFLAGS="-C prefer-dynamic -C link-arg=-Wl,-rpath,\$ORIGIN" CARGO_TARGET_DIR=target-dyn \
    cargo build -p p3-iroh-core -p p3-iroh-ping-ffi
cp target-dyn/debug/libp3_iroh_{core,ping_ffi}.so "$STD" out/p3/python/
touch out/p3/__init__.py out/p3/python/__init__.py
python3 tests/test_p3.py

# --- Design B, packaged as two separate packages, no env vars ---
RUSTFLAGS="-C prefer-dynamic -C link-arg=-Wl,-rpath,\$ORIGIN/../iroh_core" \
    CARGO_TARGET_DIR=target-rpath2 cargo build -p p3-iroh-ping-ffi
mkdir -p out/p3x/iroh_core out/p3x/iroh_ping
cp out/p3/python/p3_iroh_core.py out/p3/python/libp3_iroh_core.so "$STD" out/p3x/iroh_core/
cp out/p3/python/p3_iroh_ping_ffi.py target-rpath2/debug/libp3_iroh_ping_ffi.so out/p3x/iroh_ping/
touch out/p3x/__init__.py out/p3x/iroh_core/__init__.py out/p3x/iroh_ping/__init__.py
sed -i 's/^from \. import p3_iroh_core$/from ..iroh_core import p3_iroh_core/' \
    out/p3x/iroh_ping/p3_iroh_ping_ffi.py
env -u LD_LIBRARY_PATH python3 tests/test_p3x.py

# --- Design B for JS: two .node addons ---
RUSTFLAGS="-C prefer-dynamic" CARGO_TARGET_DIR=target-dyn \
    cargo build -p p4-napi-core -p p4-napi-ext
mkdir -p out/p4 && cp target-dyn/debug/libp4_napi_{core,ext}.so "$STD" out/p4/
LD_LIBRARY_PATH=out/p4 node tests/test_p4.js
```

Size measurements:

```sh
CARGO_PROFILE_RELEASE_LTO=true  cargo build --release -p p3-bindgen-lib   # Design A
CARGO_PROFILE_RELEASE_LTO=false CARGO_TARGET_DIR=target-nolto \
    cargo build --release -p p3-bindgen-lib -p p3-iroh-ping-ffi
RUSTFLAGS="-C prefer-dynamic" CARGO_TARGET_DIR=target-dyn \
    cargo build --release -p p3-iroh-core -p p3-iroh-ping-ffi            # Design B
```

## External references

- uniffi multi-crate support — `uniffi_bindgen::interface::ComponentInterfaces`,
  `find_component_interface(module_path)`
- `iroh-ffi/src/endpoint.rs:318` — the router assembly this design plugs into
- `iroh-ffi/make_swift.sh` — the static xcframework Design B would have to
  replace
- rustc: `-C prefer-dynamic`, and the "only shows up once" crate-uniqueness
  check that makes it safe
