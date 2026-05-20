#!/usr/bin/env python3
"""Rewrite the three version literals across the repo as a single source of truth.

Called by `cargo make prepare-release <VERSION>` in two passes:
  1. Pass 1 (just VERSION):  bumps Cargo.toml [package].version,
     iroh-js/package.json version, and Package.swift releaseTag.
  2. Pass 2 (--checksum HEX): writes Package.swift releaseChecksum once the
     deterministic xcframework zip has been built and shasum'd.

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


def bump_npm(version: str) -> None:
    p = REPO / "iroh-js" / "package.json"
    d = json.loads(p.read_text())
    d["version"] = version
    # Preserve 2-space indent + trailing newline (match existing style).
    p.write_text(json.dumps(d, indent=2) + "\n")
    print(f"  iroh-js/package.json version -> {version}")


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
    bump_npm(v)
    bump_swift_tag(v)


if __name__ == "__main__":
    main()
