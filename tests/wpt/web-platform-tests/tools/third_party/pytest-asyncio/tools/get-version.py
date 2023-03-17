import json
import sys
from importlib import metadata

from packaging.version import parse as parse_version


def main():
    version_string = metadata.version("pytest-asyncio")
    version = parse_version(version_string)
    print(f"::set-output name=version::{version}")
    prerelease = json.dumps(version.is_prerelease)
    print(f"::set-output name=prerelease::{prerelease}")


if __name__ == "__main__":
    sys.exit(main())
