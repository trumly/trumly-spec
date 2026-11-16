#!/usr/bin/env python3
"""One-off: convert the prototype's .yaml nodes into the v0 markdown format.
usage: convert.py <src nodes dir> <dest dir>"""
import sys
from pathlib import Path
import yaml

ORDER = ("id", "type", "title", "parent", "plane", "fields", "links")

def to_md(node, body=""):
    fm = {k: node[k] for k in ORDER if node.get(k) not in (None, [], {})}
    text = "---\n" + yaml.safe_dump(fm, sort_keys=False, allow_unicode=True) + "---\n"
    return text + (("\n" + body.strip() + "\n") if body.strip() else "")

if __name__ == "__main__":
    src, dst = Path(sys.argv[1]), Path(sys.argv[2])
    dst.mkdir(parents=True, exist_ok=True)
    n = 0
    for f in sorted(src.rglob("*.yaml")):
        node = yaml.safe_load(f.read_text())
        (dst / (node["id"] + ".md")).write_text(to_md(node))
        n += 1
    print(f"converted {n} nodes -> {dst}")
