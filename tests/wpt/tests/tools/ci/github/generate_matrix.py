import argparse
import json
import os
import sys
from typing import TypedDict

import jsone
import yaml

here = os.path.dirname(__file__)
config_path = os.path.join(here, "..", "chunks.yml")


# Functional form required: keys contain hyphens, which are invalid Python identifiers
MatrixEntry = TypedDict(
    "MatrixEntry",
    {
        "test-type": str,
        "current-chunk": int,
        "total-chunks": int,
        "timeout-minutes": int,
    },
)


def get_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Generate a GitHub Actions test matrix from tools/ci/chunks.yml"
    )
    parser.add_argument("--browser", required=True, help="Browser name")
    return parser


def get_matrix(browser: str, template: object = None) -> list[MatrixEntry]:
    if template is None:
        with open(config_path) as f:
            template = yaml.safe_load(f)

    config = jsone.render(template, {"browser": browser, "ci": "github-actions"})
    default_timeout = config["defaults"]["timeout"]

    includes: list[MatrixEntry] = []
    for test_type, settings in config["test_types"].items():
        total_chunks = settings["chunks"]
        if total_chunks == 0:
            continue
        timeout: int = settings.get("timeout") or default_timeout
        for chunk in range(1, total_chunks + 1):
            includes.append(
                {
                    "test-type": test_type,
                    "current-chunk": chunk,
                    "total-chunks": total_chunks,
                    "timeout-minutes": timeout,
                }
            )

    return includes


def run(venv: object, browser: str, **kwargs: object) -> None:
    json.dump({"include": get_matrix(browser)}, sys.stdout)
    print()
