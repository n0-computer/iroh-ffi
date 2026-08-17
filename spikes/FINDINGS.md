# Spike findings: `dylib` plugin linkage for iroh protocol FFI

**Both spikes PASS.** A separately-built, separately-published plugin library can share one
copy of `iroh` with the core library, and uniffi external types work across that boundary.

Environment: macOS arm64 (Darwin 25.6.0), `rustc 1.97.1 (8bab26f4f 2026-07-14)`,
uniffi 0.31.2, iroh 1.0.x, iroh-ping 1.0.0.

Reproduce:

```sh
cargo build -p iroh-ffi-ping
bash spikes/stage_python.sh debug
cd target/spike-stage && python3 round_trip.py
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
   *Every plugin repo needs this custom bindgen binary.*

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
- Each plugin repo ships a ~40-line merging bindgen — **and this is the strategic risk**:
  multi-library is explicitly unsupported upstream (finding 5). Drive a `--metadata-from`
  flag, or at least an upstream decision, before committing to the architecture.
- The compatibility gate stays in the plan, downgraded from "non-optional safety mechanism"
  to "clear diagnostics on top of an already-safe default failure".

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
