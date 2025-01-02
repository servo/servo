# mypy: disallow-untyped-defs
"""
Script used to generate a Markdown file containing only the changelog entries of a specific pytest release, which
is then published as a GitHub Release during deploy (see workflows/deploy.yml).

The script requires ``pandoc`` to be previously installed in the system -- we need to convert from RST (the format of
our CHANGELOG) into Markdown (which is required by GitHub Releases).

Requires Python3.6+.
"""

from pathlib import Path
import re
import sys
from typing import Sequence

import pypandoc


def extract_changelog_entries_for(version: str) -> str:
    p = Path(__file__).parent.parent / "doc/en/changelog.rst"
    changelog_lines = p.read_text(encoding="UTF-8").splitlines()

    title_regex = re.compile(r"pytest (\d\.\d+\.\d+\w*) \(\d{4}-\d{2}-\d{2}\)")
    consuming_version = False
    version_lines = []
    for line in changelog_lines:
        m = title_regex.match(line)
        if m:
            # Found the version we want: start to consume lines until we find the next version title.
            if m.group(1) == version:
                consuming_version = True
            # Found a new version title while parsing the version we want: break out.
            elif consuming_version:
                break
        if consuming_version:
            version_lines.append(line)

    return "\n".join(version_lines)


def convert_rst_to_md(text: str) -> str:
    result = pypandoc.convert_text(
        text, "md", format="rst", extra_args=["--wrap=preserve"]
    )
    assert isinstance(result, str), repr(result)
    return result


def main(argv: Sequence[str]) -> int:
    if len(argv) != 3:
        print("Usage: generate-gh-release-notes VERSION FILE")
        return 2

    version, filename = argv[1:3]
    print(f"Generating GitHub release notes for version {version}")
    rst_body = extract_changelog_entries_for(version)
    md_body = convert_rst_to_md(rst_body)
    Path(filename).write_text(md_body, encoding="UTF-8")
    print()
    print(f"Done: {filename}")
    print()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
