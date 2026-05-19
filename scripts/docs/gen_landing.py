#!/usr/bin/env python3
"""Generate the docs landing page (site/index.html) from support-matrix.yaml.

Deterministic transform: matrix YAML + the crate version -> one static HTML
page that links each language to its self-hosted reference docs. No model,
no network. Run via `cargo make docs-landing`.
"""
from __future__ import annotations

import html
import pathlib
import re
import sys

import yaml

REPO = pathlib.Path(__file__).resolve().parents[2]


def crate_version() -> str:
    cargo = (REPO / "Cargo.toml").read_text()
    m = re.search(r'(?m)^version\s*=\s*"([^"]+)"', cargo)
    if not m:
        sys.exit("could not find version in Cargo.toml")
    return m.group(1)


def main() -> None:
    matrix = yaml.safe_load((REPO / "support-matrix.yaml").read_text())
    langs = matrix["languages"]
    features = matrix["features"]
    version = crate_version()

    head = [f"<th>{html.escape(l['name'])}</th>" for l in langs]
    rows = []
    for feat in features:
        cells = []
        for l in langs:
            ok = feat["support"].get(l["id"], False)
            cells.append(
                f'<td class="{"yes" if ok else "no"}">{"✓" if ok else "—"}</td>'
            )
        rows.append(
            f'<tr><th scope="row">{html.escape(feat["name"])}</th>{"".join(cells)}</tr>'
        )

    cards = []
    for l in langs:
        extra = ""
        if l.get("extra"):
            e = l["extra"]
            extra = (
                f'<a class="extra" href="{html.escape(e["url"])}">'
                f'{html.escape(e["label"])} ↗</a>'
            )
        cards.append(
            f'<a class="card" href="{html.escape(l["docs"])}">'
            f"<h3>{html.escape(l['name'])}</h3>"
            f"<span>API reference →</span>{extra}</a>"
        )

    page = f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>iroh-ffi documentation</title>
<style>
  :root {{ color-scheme: light dark; }}
  body {{ font: 16px/1.5 -apple-system, system-ui, sans-serif; max-width: 880px;
         margin: 3rem auto; padding: 0 1.2rem; }}
  h1 {{ margin-bottom: .2rem; }}
  .ver {{ color: #888; margin-top: 0; }}
  .cards {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(180px,1fr));
            gap: 1rem; margin: 2rem 0; }}
  .card {{ display: block; padding: 1rem 1.2rem; border: 1px solid #8884;
           border-radius: 10px; text-decoration: none; color: inherit; }}
  .card:hover {{ border-color: #888; }}
  .card h3 {{ margin: 0 0 .3rem; }}
  .card span {{ color: #2a7ae2; font-size: .9rem; }}
  .card .extra {{ display: block; margin-top: .5rem; font-size: .8rem; color: #888; }}
  table {{ border-collapse: collapse; width: 100%; }}
  th, td {{ border: 1px solid #8884; padding: .5rem .7rem; text-align: center; }}
  th[scope=row] {{ text-align: left; }}
  td.yes {{ color: #1a7f37; font-weight: 600; }}
  td.no {{ color: #888; }}
  footer {{ margin-top: 2.5rem; color: #888; font-size: .85rem; }}
</style>
</head>
<body>
<h1>iroh-ffi</h1>
<p class="ver">iroh {html.escape(version)} &middot; bindings for Swift, Kotlin, Python &amp; JavaScript</p>
<p>A minimal FFI mirroring the iroh 1.0 API. Pick a language for its full API reference:</p>
<div class="cards">{"".join(cards)}</div>
<h2>Feature support</h2>
<table>
<thead><tr><th scope="col">Feature</th>{"".join(head)}</tr></thead>
<tbody>{"".join(rows)}</tbody>
</table>
<footer>
  Source: <a href="https://github.com/n0-computer/iroh-ffi">github.com/n0-computer/iroh-ffi</a>.
  Generated from <code>support-matrix.yaml</code>.
</footer>
</body>
</html>
"""

    out = REPO / "site" / "index.html"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(page)
    print(f"wrote {out} (iroh {version}, {len(langs)} languages, {len(features)} features)")


if __name__ == "__main__":
    main()
