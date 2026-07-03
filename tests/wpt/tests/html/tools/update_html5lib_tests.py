#!/usr/bin/env python3
"""Regenerate the html5lib_* wrapper files from the .dat files in
resources/.

The .dat files are maintained directly in-tree. To add, remove, or edit
a test, modify the .dat file and re-run this script to refresh the
wrappers' variant lists.

  update_html5lib_tests.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

TOOLS = Path(__file__).resolve().parent
WPT = TOOLS.parents[1]
PARSING = WPT / "html" / "syntax" / "parsing"
RESOURCES = PARSING / "resources"
WRAPPERS = ["url", "write", "write_single"]

DATA_BOUNDARY = re.compile(rb"(?m)^#data$")
FRAGMENT_LINE = re.compile(rb"(?m)^#document-fragment$")
SCRIPT_OFF_LINE = re.compile(rb"(?m)^#script-off$")


def classify(content: bytes) -> tuple[bool, bool]:
    """Return (has_document_test, has_fragment_test) after filtering #script-off."""
    has_doc = False
    has_frag = False
    # The first split chunk is the file preamble (before any #data); skip it.
    for chunk in DATA_BOUNDARY.split(content)[1:]:
        if SCRIPT_OFF_LINE.search(chunk):
            continue
        if FRAGMENT_LINE.search(chunk):
            has_frag = True
        else:
            has_doc = True
    return has_doc, has_frag


def main() -> int:
    # url runs everything; write/write_single only have meaning for
    # document-mode tests (fragment tests always go through innerHTML).
    runnable_url: list[str] = []
    runnable_doc: list[str] = []
    skipped: list[str] = []
    for path in sorted(RESOURCES.glob("*.dat"), key=lambda p: p.stem):
        name = path.stem
        has_doc, has_frag = classify(path.read_bytes())
        if has_doc or has_frag:
            runnable_url.append(name)
        if has_doc:
            runnable_doc.append(name)
        if not has_doc and not has_frag:
            skipped.append(name)

    print(f"  {len(runnable_url) + len(skipped)} .dat file(s) in resources/")
    if skipped:
        print(f"  not listed as variants (all #script-off): {', '.join(skipped)}")
    fragment_only = sorted(set(runnable_url) - set(runnable_doc))
    if fragment_only:
        print(f"  fragment-only (url wrapper only): {', '.join(fragment_only)}")

    for kind in WRAPPERS:
        names = runnable_url if kind == "url" else runnable_doc
        write_wrapper(PARSING / f"html5lib_{kind}.html", kind, names)
    print(f"  refreshed {len(WRAPPERS)} wrapper(s)")
    return 0


def write_wrapper(path: Path, run_type: str, names: list[str]) -> None:
    variants = "\n".join(f'<meta name="variant" content="?file={n}">' for n in names)
    path.write_text(f"""<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>HTML parser tests (run_type={run_type})</title>
<meta name="timeout" content="long">
{variants}
</head>
<body>
<h1>html5lib parser tests</h1>
<div id="log"></div>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script src="resources/common.js"></script>
<script src="resources/template.js"></script>
<script src="resources/test.js" data-run-type="{run_type}"></script>
</body>
</html>
""")


if __name__ == "__main__":
    sys.exit(main())
