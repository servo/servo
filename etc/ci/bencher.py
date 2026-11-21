#!/usr/bin/env python3

# Copyright 2024 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import argparse
import json
import os
from pathlib import Path


def size(args):
    size = os.path.getsize(args.binary)
    print(size)
    with open(args.bmf_output, "w", encoding="utf-8") as f:
        json.dump(
            {
                args.variant: {
                    "file-size": {
                        "value": float(size),
                    }
                }
            },
            f,
            indent=4,
        )


def merge(args):
    output: dict[str, object] = dict()
    for input_file in args.inputs:
        with open(input_file, "r", encoding="utf-8") as f:
            data = json.load(f)
            diff = set(data) & set(output)
            if diff:
                print("Duplicated keys:", diff)
            output = data | output

    with open(args.bmf_output, "w", encoding="utf-8") as f:
        json.dump(output, f, indent=4)


def perf_analysis(args):
    """Convert servo/perf-analysis-tools `summaries.json` files to BMF.

    The study layout is `<cpu_config>/<site>/<engine>/summaries.json`, and the
    raw durations are in seconds. Bencher's built-in `latency` measure uses
    nanoseconds, so values are multiplied by 1e9.
    """
    study_dir = Path(args.study)
    summaries_paths = sorted(study_dir.glob("*/*/*/summaries.json"))
    if not summaries_paths:
        raise SystemExit(
            f"No summaries.json found under {study_dir} (expected <cpu_config>/<site>/<engine>/summaries.json)"
        )

    output: dict[str, object] = dict()
    profile_prefix = f"{args.profile}/" if args.profile else ""
    for summaries_path in summaries_paths:
        cpu_config, site, engine = summaries_path.parent.parts[-3:]
        try:
            with open(summaries_path, "r", encoding="utf-8") as f:
                data = json.load(f)
        except (OSError, json.JSONDecodeError) as e:
            raise SystemExit(f"Failed to read {summaries_path}: {e}")

        if not isinstance(data, dict):
            raise SystemExit(f"Unexpected contents in {summaries_path}")

        for kind, key in (
            ("real", "real_events"),
            ("synthetic", "synthetic_and_interpreted_events"),
        ):
            entries = data.get(key, [])
            if not isinstance(entries, list):
                raise SystemExit(f"Expected {key} to be a list in {summaries_path}")
            for summary in entries:
                try:
                    metric_name = summary["name"]
                    raw = summary["raw"]
                    value = float(raw["mean"]) * 1e9
                    lower_value = float(raw["min"]) * 1e9
                    upper_value = float(raw["max"]) * 1e9
                except (KeyError, TypeError, ValueError) as e:
                    raise SystemExit(f"Unexpected summaries.json schema in {summaries_path}: {e}")

                name = f"{profile_prefix}PerfAnalysis/{site}/{cpu_config}/{engine}/{kind}/{metric_name}"
                if name in output:
                    raise SystemExit(f"Duplicate benchmark name: {name}")
                output[name] = {
                    "latency": {
                        "value": value,
                        "lower_value": lower_value,
                        "upper_value": upper_value,
                    }
                }

    if not output:
        raise SystemExit(f"No benchmark metrics found in {study_dir}")

    try:
        with open(args.bmf_output, "w", encoding="utf-8") as f:
            json.dump(output, f, indent=4, allow_nan=False)
    except (OSError, ValueError) as e:
        raise SystemExit(f"Failed to write {args.bmf_output}: {e}")

    print(f"Wrote {len(output)} benchmarks to {args.bmf_output}")


parser = argparse.ArgumentParser("Helper commands for bencher")

subparser = parser.add_subparsers()
size_parser = subparser.add_parser("filesize", help="Returns BMF for filesize")
size_parser.add_argument("binary", help="Servo binary file")
size_parser.add_argument("variant", help="variant of the binary")
size_parser.add_argument("--bmf-output", help="BMF JSON output file", default=None)
size_parser.set_defaults(func=size)

merge_parser = subparser.add_parser("merge", help="Merges BMF JSONs")
merge_parser.add_argument("--bmf-output", help="BMF JSON output file")
merge_parser.add_argument("inputs", help="BMF JSON files to merge", nargs="+")
merge_parser.set_defaults(func=merge)

perf_analysis_parser = subparser.add_parser("perf-analysis", help="Converts perf-analysis-tools summaries to BMF")
perf_analysis_parser.add_argument("study", help="perf-analysis study directory containing summaries.json files")
perf_analysis_parser.add_argument("--profile", default="", help="Build profile prefix for benchmark names")
perf_analysis_parser.add_argument("--bmf-output", required=True, help="BMF JSON output file")
perf_analysis_parser.set_defaults(func=perf_analysis)

args = parser.parse_args()
args.func(args)
