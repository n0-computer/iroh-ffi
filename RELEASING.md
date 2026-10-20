# Releasing iroh-ffi

Pushing a `v<VERSION>` tag publishes every binding:

| Binding | Published as | Workflow |
|---|---|---|
| JavaScript | `@number0/iroh` on npm | `ci_js.yml` (`publish` job) |
| Python | `iroh` on PyPI | `wheels.yml` (`publish` job) |
| Kotlin | `computer.iroh:iroh` and `computer.iroh:iroh-android` on Maven Central | `release.yml` |
| Swift | `IrohLib.xcframework.zip` on the GitHub release, referenced by `Package.swift` | built by `release_swift.yml`, published by `release.yml` |
| C | `libiroh-<target>` static library archives on the GitHub release | `release.yml` |
| Docs | API docs for all bindings on GitHub Pages | `docs.yml` |

Version literals are written locally by `cargo make prepare-release`. The Swift
checksum in `Package.swift` is written by CI on the release branch. CI never
writes to `main`.

## Choosing the version

Use semver: patch for fixes only, minor for new API, major for breaking
changes. A pre-release version such as `1.2.0-rc.1` publishes to npm under the
`next` dist-tag instead of `latest`.

## Prerequisites

- `cargo`, `yarn` and Python 3.
- Push access to the repository.
- `main` is green at the commit you release.
- The self-hosted runners are online, including the macOS runner labelled
  `xcode16`.

## 1. Dry run

**On CI.** Builds the C libraries and the Kotlin native libraries, and runs the
Android emulator test. Publish steps in `release.yml` only run on tag refs, so
nothing is published:

```sh
gh workflow run release.yml --ref main
```

In the log of the "Smoke Kotlin Android consumer (emulator)" job, check for
four `OK: … (16 KB LOAD alignment)` lines, one per Android ABI.

> [!WARNING]
> Do not start `ci_js.yml`, `wheels.yml` or `docs.yml` manually as a dry run.
> A manual run of `ci_js.yml` or `wheels.yml` publishes the branch's version
> to npm or PyPI. A manual run of `docs.yml` deploys the docs site.

**Locally (optional, macOS only).** Needs Xcode with an iOS Simulator runtime
and JDK 17 (`brew install --cask temurin@17`):

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios \
  aarch64-apple-darwin aarch64-apple-ios-macabi
# armored private key, same as the SIGNING_KEY secret
export ORG_GRADLE_PROJECT_signingInMemoryKey="$(cat ~/.keys/maven_key.priv.asc)"
cargo make pre-release-check
```

This builds and checks the Swift xcframework, the Python wheel, the npm package
(`npm publish --dry-run`) and the signed Maven publication
(`publishToMavenLocal`). The Swift zip built here is only a check; the
published zip comes from CI (step 3).

The key is needed because both Kotlin modules sign every publication, local
ones included. This makes the local dry run the only check of Maven signing
before the tag, since the CI dry run skips the Maven Central publish. It checks
your local copy of the key, not the `SIGNING_KEY` secret.

## 2. Open the release PR

```sh
git checkout main && git pull
cargo make prepare-release 1.2.0     # no leading v
git checkout -b release/v1.2.0
git commit -am "chore(release): v1.2.0"
git push -u origin release/v1.2.0
```

Open a PR against `main`. Expect changes in these files:

- `Cargo.toml`, `Cargo.lock`
- `iroh-js/Cargo.toml`, `iroh-js/package.json`, `iroh-js/npm/*/package.json`,
  `iroh-js/yarn.lock`, `iroh-js/index.js`
- `pyproject.toml`
- `kotlin/lib/build.gradle.kts`, `kotlin/android/build.gradle.kts`
- `Package.swift` (`releaseTag` only)

## 3. Let CI build the Swift artifact

Every push to a `release/v*` branch runs `release_swift.yml`. It:

1. builds the xcframework and runs the layout and iOS Simulator checks;
2. commits `chore(release): bake Swift xcframework SHA <hex> [skip swift-release]`
   to the branch, writing the zip's checksum into `Package.swift`;
3. uploads `IrohLib.xcframework.zip` to a draft GitHub release `v<VERSION>`.

Each later push rebuilds the zip and adds a new bot commit, so run `git pull`
before pushing to the branch again.

Before merging, check that `release_swift.yml` succeeded for your last push
and that the draft release has the zip attached.

## 4. Merge and tag

Merge the PR, then tag the merge commit:

```sh
git checkout main && git pull
git tag v1.2.0 <merge-commit-sha>
git push origin v1.2.0
```

## 5. Watch the publish runs

| Workflow | Does |
|---|---|
| `release.yml` | Publishes the draft GitHub release, attaches the C library archives, runs the Android emulator test, publishes both Kotlin artifacts to Maven Central |
| `ci_js.yml` | Runs the JS test matrix, then publishes to npm with provenance (Trusted Publishing) |
| `wheels.yml` | Builds the wheels and publishes them to PyPI (Trusted Publishing) |
| `docs.yml` | Rebuilds the API docs and deploys GitHub Pages |

`release.yml` does not rebuild the Swift zip. It publishes the draft from step
3, so the checksum in `Package.swift` matches the published zip.

## 6. Check the published packages

Install each package from its registry, not from a local build:

```sh
V=1.2.0
gh release view "v$V"    # not a draft; IrohLib.xcframework.zip and four libiroh-* archives

cd "$(mktemp -d)" && npm init -y >/dev/null && npm install "@number0/iroh@$V"
node --throw-deprecation -e "require('@number0/iroh')"

cd "$(mktemp -d)" && python3 -m venv venv && venv/bin/pip install "iroh==$V"
venv/bin/python -c "import iroh"

curl -s https://repo1.maven.org/maven2/computer/iroh/iroh-android/maven-metadata.xml | grep "$V"
```

For Swift, depend on
`.package(url: "https://github.com/n0-computer/iroh-ffi", exact: "<VERSION>")`
in a scratch package and run `swift package resolve`. SwiftPM downloads the zip
and verifies its checksum.

Maven Central can take some time before a new version is visible.

## If a publish fails

- **Temporary failure** (runner offline, network error): re-run the failed jobs
  in the tag's workflow run.
- **Failure that needs a code or workflow change:** fix it on `main` and
  release the next patch version. Do not move or re-push an existing tag.
  Registries refuse to publish a version twice, and SwiftPM records the commit
  of each version it resolves and rejects that version once its tag points
  elsewhere.

## Kotlin artifacts

- `computer.iroh:iroh` is the JVM JAR. It bundles the desktop native libraries
  (Linux x86_64 and aarch64, macOS aarch64, Windows x86_64), loaded by JNA.
- `computer.iroh:iroh-android` is the AAR. It bundles `libiroh_ffi.so` for each
  Android ABI and the `IrohAndroid` initializer, and depends on
  `computer.iroh:iroh` for the Kotlin API.

Android apps depend only on the AAR:

```kotlin
implementation("computer.iroh:iroh-android:<VERSION>")
```

## Why CI builds the Swift zip on the release branch

SwiftPM requires the checksum in `Package.swift` to match the downloaded zip
byte for byte. Building the zip locally and again on tag CI required both
builds to be identical across machines, which kept failing in practice.
Building once on the release branch and publishing that exact draft at tag
time removes the problem.

## One-time setup

- **npm:** Trusted Publishing configured at npmjs.com. No token.
- **PyPI:** a trusted publisher at <https://pypi.org/manage/account/publishing/>
  with project `iroh`, owner `n0-computer`, repository `iroh-ffi`, workflow
  `wheels.yml`.
- **Maven Central:** the `computer.iroh` namespace, verified by a DNS TXT record
  on `iroh.computer`. Repository secrets `SONATYPE_USERNAME` and
  `SONATYPE_PASSWORD` (Sonatype Central credentials) and `SIGNING_KEY`
  (armored private PGP key).
