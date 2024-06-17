from subprocess import call
import sys


def main():
    """
    Platform agnostic wrapper script for towncrier.
    Fixes the issue (pytest#7251) where windows users are unable to natively
    run tox -e docs to build pytest docs.
    """
    with open("docs/_changelog_towncrier_draft.rst", "w") as draft_file:
        return call(("towncrier", "--draft"), stdout=draft_file)


if __name__ == "__main__":
    sys.exit(main())
