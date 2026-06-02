# mypy: disallow-untyped-defs
from subprocess import call
import sys


def main() -> int:
    """
    Platform-agnostic wrapper script for towncrier.
    Fixes the issue (#7251) where Windows users are unable to natively run tox -e docs to build pytest docs.
    """
    with open(
        "doc/en/_changelog_towncrier_draft.rst", "w", encoding="utf-8"
    ) as draft_file:
        return call(("towncrier", "--draft"), stdout=draft_file)


if __name__ == "__main__":
    sys.exit(main())
