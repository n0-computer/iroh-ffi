# Releasing iroh-ffi

A release ships a new version of the iroh-ffi crate's surface to every
binding host (npm, PyPI, GitHub Releases for Swift) plus refreshed docs on
GitHub Pages. The tag is the trigger — pushing `v<VERSION>` fans out to all
four publish workflows independently.

The repo manifest stays **self-consistent at every commit**: the version
literals in `Cargo.toml`, `iroh-js/package.json`, and `Package.swift` (tag +
checksum) are written by a local script before the release PR is opened. CI
never writes to `main`.

## Prerequisites

- macOS workstation with Xcode (the prepare-release script builds the Apple
  xcframework to compute the SwiftPM checksum).
- Push rights on the repo + the relevant publisher accounts (npm
  `@number0/iroh`; PyPI trusted publisher for `iroh`; Sonatype Central for
  `computer.iroh` once Maven publishing is wired).
- `main` is green at the commit you want to release from.

## Steps

0. **Local dry-run** (recommended). Validates every artifact builds without
   touching any registry:

   ```sh
   # optional: dry-run signing too
   export ORG_GRADLE_PROJECT_signingInMemoryKey="$(gpg --export-secret-keys --armor <KEY_ID>)"

   cargo make pre-release-check
   ```

   Builds the Swift xcframework + deterministic zip, the Python wheel,
   the napi addon + `npm publish --dry-run`, and `gradle
   publishToMavenLocal`. Catches config typos, signing failures, manifest
   errors before the real publish.

1. **Prepare the release branch + commit.**

   ```sh
   git checkout main && git pull
   cargo make prepare-release 1.0.0-rc.1     # no leading v
   ```

   This rewrites the version literals, refreshes lockfiles, builds a
   deterministic `IrohLib.xcframework.zip`, and bakes its SHA-256 into
   `Package.swift`. Inspect the diff (`git diff`) — expected files only:
   `Cargo.toml`, `Cargo.lock`, `iroh-js/package.json`, `iroh-js/yarn.lock`,
   `Package.swift`.

2. **Open the release PR.**

   ```sh
   git checkout -b release/v1.0.0-rc.1
   git commit -am "chore(release): v1.0.0-rc.1"
   git push -u origin release/v1.0.0-rc.1
   gh pr create --base main --fill --title "chore(release): v1.0.0-rc.1"
   ```

   Get all CI green on the PR.

3. **Merge to `main`.** Squash or merge-commit — both work; whichever lands
   keeps Package.swift consistent.

4. **Tag the merge commit and push.**

   ```sh
   git checkout main && git pull
   git tag v1.0.0-rc.1
   git push origin v1.0.0-rc.1
   ```

5. **Watch the fan-out.** The tag triggers:

   | Workflow | Publishes |
   |---|---|
   | `release.yml` | GitHub Release + per-OS binary assets + `IrohLib.xcframework.zip` |
   | `ci_js.yml` `publish` | `@number0/iroh` to npm (`--provenance`, `NPM_TOKEN`) |
   | `wheels.yml` `publish` | `iroh` wheels to PyPI (Trusted Publishing / OIDC — no token) |
   | `docs.yml` | GitHub Pages site refresh |
   | _(future)_ `release.yml` `build-and-publish-kotlin` | Maven Central `computer.iroh:iroh` (blocked on #147) |

   If any publish fails, fix and re-tag with the next patch version (do not
   re-use a tag).

## Why local-only prepare

Binary SwiftPM packages need the manifest checksum to match the uploaded
zip byte-for-byte. We use a deterministic zip recipe
(`scripts/release/zip_xcframework.sh`: fixed mtimes + sorted file list + no
zip extra fields) so the local zip and the one rebuilt by `release.yml`
hash identically. This lets `Package.swift` carry the correct checksum
from the release commit forward — no CI write-back to `main`, no
intermediate broken state. Matches the firebase-ios-sdk / Sentry-Cocoa
release pattern for binary SwiftPM packages.

## One-time publisher setup (per host)

- **npm** — `NPM_TOKEN` repo secret with publish rights on `@number0/iroh`.
- **PyPI** — register a trusted publisher at
  <https://pypi.org/manage/account/publishing/>: project `iroh`, owner
  `n0-computer`, repository `iroh-ffi`, workflow `wheels.yml`. No long-lived
  token needed. Until registered, the first publish job will fail loudly.
- **Maven Central** _(future, blocked on #147)_ — claim `computer.iroh`
  namespace via DNS TXT on `iroh.computer`; GPG signing key; OSSRH creds.
