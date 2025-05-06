import argparse
import os
from typing import Optional, Set

import requests

from ..manifest import manifest
from ..wpt import metadata, virtualenv

here = os.path.dirname(__file__)
wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))


def get_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", dest="manifest_path", help="Path to MANIFEST.json")
    parser.add_argument("metadata_path", help="Path to wpt metadata repository")
    return parser


def get_labels(interop_year: int = 2025) -> Set[str]:
    data_url = "https://raw.githubusercontent.com/web-platform-tests/wpt.fyi/refs/heads/main/webapp/static/interop-data.json"
    resp = requests.get(data_url)
    resp.raise_for_status()
    data = resp.json()
    focus_areas = data[str(interop_year)]["focus_areas"]
    labels = set()
    for focus_area in focus_areas.values():
        labels |= set(focus_area["labels"])

    return labels


def run_update_codeowners(venv: virtualenv.Virtualenv,
                          metadata_path: str,
                          manifest_path: Optional[str],
                          interop_year: int = 2025,
                          reviewer: Optional[str] = "@web-platform-tests/interop") -> int:
    if manifest_path is None:
        manifest_path = os.path.join(wpt_root, "MANIFEST.json")
    wpt_manifest = manifest.load_and_update(wpt_root, manifest_path, "/")

    labels = get_labels(interop_year)
    metadata_map = metadata.load_metadata_map(metadata_path)
    tests_by_label = metadata.get_labelled_tests(metadata_map,
                                                 list(labels))
    all_labelled_tests = set()
    for labelled_tests in tests_by_label.values():
        all_labelled_tests |= set(labelled_tests)

    test_lines = []
    for _test_type, rel_path, tests in wpt_manifest:
        if any(test.id in all_labelled_tests for test in tests):
            test_lines.append(f"{rel_path} {reviewer}\n")

    output = []
    start_line = "# GENERATED: interop-tests"
    end_line = "# END GENERATED"
    in_generated = False
    found_generated = False
    with open(os.path.join(wpt_root, "CODEOWNERS")) as f:
        for line in f:
            if not in_generated:
                if line.strip() == start_line:
                    output.append(line)
                    found_generated = True
                    in_generated = True
                    output.extend(test_lines)
                else:
                    output.append(line)
            else:
                if line.strip() == end_line:
                    in_generated = False
                    output.append(line)
    if not found_generated:
        output.extend(["\n", start_line + "\n"] + test_lines + [end_line + "\n"])
    with open(os.path.join(wpt_root, "CODEOWNERS"), "w") as f:
        f.writelines(output)

    return 0
