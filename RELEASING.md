# Releasing iroh-ffi

A release ships a new version of the iroh-ffi crate's surface to every binding
host (npm, PyPI, Maven Central, GitHub Releases for Swift) plus refreshed docs
on GitHub Pages. The tag is the trigger — pushing `v<VERSION>` fans out to all
publish workflows independently.

The repo manifest stays self-consistent at every commit. Version literals in
`Cargo.toml`, `iroh-js/package.json`, etc. are written by a local script
before the release PR is opened; the Swift xcframework SHA-256 in
`Package.swift` is written by PR CI (one bot commit on the release branch).
CI never writes to `main`.

## Prerequisites

- Local shell with `cargo`, `yarn`, and Python 3 (no Xcode needed —
  prepare-release no longer builds the xcframework).
- Push rights on the repo + the relevant publisher accounts (npm
  `@number0/iroh`; PyPI trusted publisher for `iroh`; Sonatype Central for
  `computer.iroh`).
- `main` is green at the commit you want to release from.

## Steps

0. **Local dry-run** (recommended, macOS only — needs Xcode for the swift slot).
   Validates every artifact builds without touching any registry:

   ```sh
   # the armored private key (same contents as the SIGNING_KEY CI secret)
   export ORG_GRADLE_PROJECT_signingInMemoryKey="$(cat ~/.keys/maven_key.sec.asc)"

   cargo make pre-release-check
   ```

   Builds the Swift xcframework (sanity check only — the published zip
   comes from PR CI, not here), the Python wheel + import smoke, the napi
   addon + `npm publish --dry-run`, and `gradle publishToMavenLocal` +
   JNA-from-per-platform-staged smoke. Catches config typos, signing
   failures, manifest errors before the real publish.

1. **Bump versions on a release branch.**

   ```sh
   git checkout main && git pull
   cargo make prepare-release 1.0.0-rc.1     # no leading v
   git checkout -b release/v1.0.0-rc.1
   git commit -am "chore(release): v1.0.0-rc.1"
   git push -u origin release/v1.0.0-rc.1
   gh pr create --base main --fill --title "chore(release): v1.0.0-rc.1"
   ```

   `prepare-release` only rewrites version literals + lockfiles. Inspect
   the diff (`git diff`) — expected files only: `Cargo.toml`, `Cargo.lock`,
   `iroh-js/package.json` + `iroh-js/Cargo.toml`, `iroh-js/yarn.lock`,
   `pyproject.toml`, `kotlin/lib/build.gradle.kts`, `Package.swift`
   (releaseTag only — the checksum is filled in by CI in step 2).

2. **Wait for PR CI to bake the Swift xcframework SHA.**

   `release_swift.yml` fires on push to `release/v*` branches. It builds
   the xcframework on a self-hosted macOS runner, runs the verify gates
   (layout + iOS sim test), zips deterministically, and:
   - commits `chore(release): bake Swift xcframework SHA <hex>
     [skip swift-release]` to the release branch (overwrites the
     placeholder checksum in `Package.swift`);
   - uploads `IrohLib.xcframework.zip` to a draft GitHub release named
     `v1.0.0-rc.1`.

   Review the bot's commit + the draft release asset (downloadable from
   the GitHub Releases page) before merging. The SHA in `Package.swift`
   is exactly the SHA of the zip on the draft release — no cross-host
   determinism game.

3. **Merge to `main`.** Squash or merge-commit; both keep `Package.swift`
   consistent with the draft release.

4. **Tag the merge commit and push.**

   ```sh
   git checkout main && git pull
   git tag v1.0.0-rc.1
   git push origin v1.0.0-rc.1
   ```

5. **Watch the fan-out.** The tag triggers:

   | Workflow | Publishes |
   |---|---|
   | `release.yml` | promotes the draft GH release v<version> to published; uploads per-OS C lib archives |
   | `ci_js.yml` `publish` | `@number0/iroh` to npm (`--provenance`, OIDC) |
   | `wheels.yml` `publish` | `iroh` wheels to PyPI (OIDC) |
   | `release.yml` `build-and-publish-kotlin` | `computer.iroh:iroh` (JVM JAR) + `computer.iroh:iroh-android` (AAR) to Maven Central — one `./gradlew publishAndReleaseToMavenCentral` invocation, both subprojects published in the same Sonatype staging |
   | `docs.yml` | GitHub Pages site refresh |

   The Swift xcframework zip is **not** rebuilt at tag time — `release.yml`
   `create-release` just promotes the PR-CI-built draft.

   If any publish fails, fix and re-tag with the next patch version (do
   not re-use a tag).

## Kotlin: two artifacts, one publish

The release publishes both `computer.iroh:iroh` (JVM JAR, for desktop
Java/Kotlin consumers — JNA-loaded desktop natives only) and
`computer.iroh:iroh-android` (AAR, for Android consumers — bundles
`libiroh_ffi.so` per ABI at `jni/<abi>/` plus the `IrohAndroid` initializer
class). The AAR `api`-depends on the JAR, so an Android consumer that pulls
in `iroh-android` transitively gets the full Kotlin API surface.

**Migrating an Android consumer from `iroh` → `iroh-android`:** one-line
dependency swap. `IrohAndroid` moved from `computer.iroh` (in the JAR) to
`computer.iroh.android` … actually still `computer.iroh` (the package
didn't change, only which artifact ships the class). Imports stay
unchanged; only the Gradle coordinate moves:

```kotlin
// before
implementation("computer.iroh:iroh:<VERSION>")

// after (Android consumers — apps + library modules with
// com.android.application / com.android.library)
implementation("computer.iroh:iroh-android:<VERSION>")
```

JVM consumers stay on `computer.iroh:iroh` and see a smaller JAR (no
Android natives bundled in).

## Why PR-CI builds the Swift xcframework (not local, not tag-CI)

Binary SwiftPM packages need the manifest checksum to match the uploaded
zip byte-for-byte. The previous flow built the xcframework locally during
`prepare-release`, baked the SHA into `Package.swift`, then rebuilt on
tag-CI and expected the SHA to still match — which required cross-host
build determinism (matching Xcode patch version, macOS SDK, rustc, cargo
deps, etc.). That kept failing in practice.

Single source of truth: PR CI builds the zip once, bakes its SHA into the
PR, uploads to a draft release. Tag CI just promotes the draft. The SHA
in `Package.swift` is the SHA of the bytes on the release — they were
literally the same zip. No build-environment skew can break consumers.

The trade-off is that the release branch picks up one bot commit
(`bake Swift xcframework SHA <hex>`), which is reviewable in the PR diff.
`[skip swift-release]` in the commit message prevents the workflow from
re-firing on its own output.

## One-time publisher setup (per host)

- **npm** — Trusted Publishing configured at npmjs.com (no token).
- **PyPI** — register a trusted publisher at
  <https://pypi.org/manage/account/publishing/>: project `iroh`, owner
  `n0-computer`, repository `iroh-ffi`, workflow `wheels.yml`.
- **Maven Central** — `computer.iroh` namespace claimed via DNS TXT on
  `iroh.computer`; armored PGP key in `SIGNING_KEY` secret; Sonatype
  Central API creds.
