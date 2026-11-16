#!/usr/bin/env python3
"""Regenerate expected.json for every case from the Python reference. Review the git diff before committing."""
import json, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "reference"))
import core
for d in sorted((ROOT / "fixtures" / "cases").iterdir()):
    if (d / "nodes").is_dir():
        out = core.check(d / "nodes")
        (d / "expected.json").write_text(json.dumps(out, indent=1, ensure_ascii=False, sort_keys=True) + "\n")
        print("blessed", d.name)
