#!/usr/bin/env python3
"""Rewrite the version literals across the repo as a single source of truth.

Two entry points:
  1. `cargo make prepare-release <V>` calls this with just VERSION to bump
     Cargo.toml [package].version, iroh-js/{Cargo.toml,package.json}, all the
     iroh-js/npm/*/package.json sub-packages, pyproject.toml,
     kotlin/lib/build.gradle.kts coordinates, and Package.swift releaseTag.
  2. release_swift.yml (PR CI) calls this with --checksum HEX once it has
     built the xcframework and shasum'd the deterministic zip, to write
     Package.swift releaseChecksum.

Pure deterministic text/JSON transforms — no model, no network (org Rule 5).
"""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys

REPO = pathlib.Path(__file__).resolve().parents[2]


def bump_cargo(version: str) -> None:
    p = REPO / "Cargo.toml"
    s = p.read_text()
    # Match `version = "..."` inside the [package] block only — not workspace
    # deps or any other [section]. Anchor on `[package]` then forbid `[`.
    new, n = re.subn(
        r'(?ms)(\[package\][^\[]*?)\nversion = "[^"]+"',
        lambda m: m.group(1) + f'\nversion = "{version}"',
        s,
        count=1,
    )
    if n != 1:
        sys.exit("could not find [package].version in Cargo.toml")
    p.write_text(new)
    print(f"  Cargo.toml [package].version -> {version}")


def bump_js_cargo(version: str) -> None:
    # The napi crate (iroh-js/Cargo.toml, package `number0_iroh`) is a separate
    # [package] from the root workspace member — `bump_cargo()` doesn't touch it.
    p = REPO / "iroh-js" / "Cargo.toml"
    s = p.read_text()
    new, n = re.subn(
        r'(?ms)(\[package\][^\[]*?)\nversion = "[^"]+"',
        lambda m: m.group(1) + f'\nversion = "{version}"',
        s,
        count=1,
    )
    if n != 1:
        sys.exit("could not find [package].version in iroh-js/Cargo.toml")
    p.write_text(new)
    print(f"  iroh-js/Cargo.toml [package].version -> {version}")


def bump_pyproject(version: str) -> None:
    # maturin reads [project].version from pyproject.toml when building the
    # wheel — if this is stale, PyPI gets the wrong version (filename + metadata)
    # even though every other manifest is up to date.
    p = REPO / "pyproject.toml"
    s = p.read_text()
    new, n = re.subn(
        r'(?ms)(\[project\][^\[]*?)\nversion = "[^"]+"',
        lambda m: m.group(1) + f'\nversion = "{version}"',
        s,
        count=1,
    )
    if n != 1:
        sys.exit("could not find [project].version in pyproject.toml")
    p.write_text(new)
    print(f"  pyproject.toml [project].version -> {version}")


def bump_npm(version: str) -> None:
    # Main package.json: bump "version". Do NOT add/maintain optionalDependencies
    # in source — `napi pre-publish` writes that block at publish time (referencing
    # versions that don't exist on npm yet would break `yarn install` on PR CI).
    main_p = REPO / "iroh-js" / "package.json"
    main_d = json.loads(main_p.read_text())
    main_d["version"] = version
    main_p.write_text(json.dumps(main_d, indent=2) + "\n")
    print(f"  iroh-js/package.json version -> {version}")

    # Per-target sub-packages under iroh-js/npm/<target>/package.json — each one is
    # published as a separate npm package (@number0/iroh-<target>) and must carry
    # the same version, or the main package's optionalDependencies won't resolve.
    sub_pkgs = sorted((REPO / "iroh-js" / "npm").glob("*/package.json"))
    if not sub_pkgs:
        sys.exit("no iroh-js/npm/*/package.json sub-packages found")
    for sp in sub_pkgs:
        sd = json.loads(sp.read_text())
        sd["version"] = version
        # Sub-packages are written WITHOUT a trailing newline (matches napi-rs output).
        sp.write_text(json.dumps(sd, indent=2))
    print(f"  iroh-js/npm/*/package.json version -> {version} ({len(sub_pkgs)} sub-packages)")


def bump_gradle(version: str) -> None:
    # Two coordinate literals to bump: :lib publishes computer.iroh:iroh (the
    # JVM JAR), :android publishes computer.iroh:iroh-android (the AAR). They
    # must move together — :android depends on the matching :lib version.
    for sub, artifact in (("lib", "iroh"), ("android", "iroh-android")):
        p = REPO / "kotlin" / sub / "build.gradle.kts"
        s = p.read_text()
        new, n = re.subn(
            rf'coordinates\("computer\.iroh", "{re.escape(artifact)}", "[^"]+"\)',
            f'coordinates("computer.iroh", "{artifact}", "{version}")',
            s,
            count=1,
        )
        if n != 1:
            sys.exit(f'could not find coordinates("computer.iroh", "{artifact}", "...") in kotlin/{sub}/build.gradle.kts')
        p.write_text(new)
        print(f"  kotlin/{sub}/build.gradle.kts coordinates -> {version}")


def bump_swift_tag(version: str) -> None:
    p = REPO / "Package.swift"
    s = p.read_text()
    new, n = re.subn(
        r'let releaseTag = "[^"]+"',
        f'let releaseTag = "v{version}"',
        s,
        count=1,
    )
    if n != 1:
        sys.exit("could not find releaseTag in Package.swift")
    p.write_text(new)
    print(f'  Package.swift releaseTag -> "v{version}"')


def write_swift_checksum(checksum: str) -> None:
    if not re.fullmatch(r"[0-9a-f]{64}", checksum):
        sys.exit(f"checksum {checksum!r} is not a 64-char lowercase hex SHA-256")
    p = REPO / "Package.swift"
    s = p.read_text()
    new, n = re.subn(
        r'let releaseChecksum = "[^"]+"',
        f'let releaseChecksum = "{checksum}"',
        s,
        count=1,
    )
    if n != 1:
        sys.exit("could not find releaseChecksum in Package.swift")
    p.write_text(new)
    print(f"  Package.swift releaseChecksum -> {checksum}")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("version", nargs="?", help='e.g. "1.0.0-rc.1" (no leading v)')
    ap.add_argument("--checksum", help="64-char SHA-256 hex; writes only releaseChecksum")
    args = ap.parse_args()

    if args.checksum and args.version:
        sys.exit("pass either VERSION or --checksum, not both")
    if not args.checksum and not args.version:
        ap.error("VERSION or --checksum required")

    if args.checksum:
        write_swift_checksum(args.checksum)
        return

    v = args.version.lstrip("v")
    # Permissive semver (incl. pre-release like 1.0.0-rc.1).
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?", v):
        sys.exit(f"{v!r} is not a recognized semver string")
    print(f"bumping versions to {v}:")
    bump_cargo(v)
    bump_js_cargo(v)
    bump_pyproject(v)
    bump_npm(v)
    bump_gradle(v)
    bump_swift_tag(v)


if __name__ == "__main__":
    main()
