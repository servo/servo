#!/usr/bin/env python3
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""
Embedding API observability reporter (W5.6).

Collects the per-platform metric files written by the C API integration tests
(see `docs/embedding/OBSERVABILITY.md`), compares them against the committed
baseline at `docs/embedding/observability_baseline.json`, and prints a markdown
table.

This is a *reporting* job, not a gate: hosted-runner timings move by tens of
percent between runs, so a strict threshold here would be noise that fails
builds. The job therefore always exits 0 for measured regressions. It does exit
non-zero for malformed input, because that means the collection itself broke.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timezone
from typing import Any

# metric key, human label, unit
METRICS: list[tuple[str, str, str]] = [
    ("startup_ms", "engine startup", "ms"),
    ("load_complete_ms", "load complete", "ms"),
    ("screenshot_ms", "paint + screenshot", "ms"),
    ("js_ms", "fixed JS workload", "ms"),
    ("cdylib_bytes", "cdylib size", "bytes"),
]

# A delta beyond this fraction is flagged in the table (never enforced).
FLAG_RATIO = 0.25


def load_metrics(directory: str) -> dict[str, dict[str, Any]]:
    """Loads every metric file under `directory`, keyed by platform.

    The search is recursive because `actions/download-artifact` gives each
    artifact a directory of its own, so the files arrive nested rather than
    directly in `directory` (which is what the first version of this script
    assumed, and it silently reported "no metrics" in CI).
    """
    platforms: dict[str, dict[str, Any]] = {}
    if not os.path.isdir(directory):
        return platforms

    for root, _directories, names in os.walk(directory):
        for name in sorted(names):
            # `summary.json` is this script's own output, not a measurement.
            if not name.endswith(".json") or name == "summary.json":
                continue
            path = os.path.join(root, name)
            with open(path, "r", encoding="utf-8") as handle:
                data = json.load(handle)
            if not isinstance(data, dict):
                raise ValueError(f"{path}: expected a JSON object")

            platform = data.pop("platform", os.path.splitext(name)[0])
            platforms[str(platform)] = data

    return platforms


def load_baseline(path: str) -> dict[str, Any] | None:
    """Loads the committed baseline, or returns None when it does not exist yet."""
    if not os.path.isfile(path):
        return None
    with open(path, "r", encoding="utf-8") as handle:
        data = json.load(handle)
    if not isinstance(data, dict):
        raise ValueError(f"{path}: expected a JSON object")
    return data


def format_value(value: Any, unit: str) -> str:
    if value is None:
        return "-"
    if unit == "bytes":
        return f"{value / 1024:.1f} KiB"
    return f"{value} ms"


def delta_cell(value: Any, base: Any) -> str:
    if value is None or base in (None, 0):
        return "-"
    ratio = (value - base) / base
    if abs(ratio) < 0.005:
        return "same"
    marker = " (!)" if abs(ratio) > FLAG_RATIO else ""
    return f"{ratio:+.1%}{marker}"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metrics", required=True,
                        help="directory holding the per-platform metric JSON files")
    parser.add_argument("--baseline", required=True,
                        help="path to the committed baseline JSON")
    parser.add_argument("--summary", required=True,
                        help="path of the markdown summary to write")
    parser.add_argument("--out", required=True,
                        help="path of the merged JSON report to write")
    parser.add_argument("--capi-result", default="unknown",
                        help="conclusion of the C API test job (the crash proxy)")
    args = parser.parse_args()

    platforms = load_metrics(args.metrics)
    baseline = load_baseline(args.baseline)
    baseline_platforms = (baseline or {}).get("platforms", {})

    lines: list[str] = [
        "## Embedding API observability",
        "",
        f"- Metrics directory: `{args.metrics}`",
        f"- C API test job result (crash proxy): **{args.capi_result}**",
        "",
    ]

    report: dict[str, Any] = {
        "generated_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "capi_result": args.capi_result,
        "platforms": platforms,
        "baseline_platforms": baseline_platforms,
    }

    if not platforms:
        lines.append("No metric files were found. The collecting job either did "
                     "not run or failed before it wrote them.")
        if args.capi_result == "success":
            lines.append("")
            lines.append("The C API test job **succeeded**, so the metrics it was "
                         "supposed to write are missing for a reason other than a "
                         "crash: failing this job is the point, otherwise the "
                         "observability job reports success while measuring nothing.")
    else:
        header = [metric[1] for metric in METRICS]
        lines.append("| Platform | " + " | ".join(header) + " |")
        lines.append("| --- |" + " --- |" * len(header))
        for platform, values in sorted(platforms.items()):
            cells = [format_value(values.get(key), unit) for key, _label, unit in METRICS]
            lines.append(f"| {platform} | " + " | ".join(cells) + " |")

        if baseline_platforms:
            lines.append("")
            lines.append("### Change against the baseline")
            lines.append("")
            lines.append("| Platform | " + " | ".join(header) + " |")
            lines.append("| --- |" + " --- |" * len(header))
            for platform, values in sorted(platforms.items()):
                base = baseline_platforms.get(platform)
                if not isinstance(base, dict):
                    lines.append(f"| {platform} |" + " (no baseline) |" * len(header))
                    continue
                cells = [delta_cell(values.get(key), base.get(key)) for key, _label, _unit in METRICS]
                lines.append(f"| {platform} | " + " | ".join(cells) + " |")
            lines.append("")
            lines.append(f"A `(!)` marks a change of more than {FLAG_RATIO:.0%}. "
                         "This job does not fail on it: see "
                         "`docs/embedding/OBSERVABILITY.md` for the thresholds a "
                         "real gate would need.")

    if baseline is None:
        lines.append("")
        lines.append("**There is no committed baseline yet.** Copy the measured "
                     "values from the table above into "
                     "`docs/embedding/observability_baseline.json` (same shape as "
                     "the `platforms` object reported here) so the next run shows "
                     "a diff.")

    summary = "\n".join(lines) + "\n"
    os.makedirs(os.path.dirname(args.summary) or ".", exist_ok=True)
    with open(args.summary, "w", encoding="utf-8") as handle:
        handle.write(summary)

    report["markdown"] = summary
    os.makedirs(os.path.dirname(args.out) or ".", exist_ok=True)
    with open(args.out, "w", encoding="utf-8") as handle:
        json.dump(report, handle, indent=2, sort_keys=True)
        handle.write("\n")

    print(summary)

    # Reporting never fails on a *measured* regression (see the module docstring),
    # but collecting nothing while the job that produces the metrics succeeded
    # means the pipeline is broken, and a green job that measures nothing is
    # worse than a red one.
    if not platforms and args.capi_result == "success":
        print("error: no metric files found although the C API job succeeded",
              file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
