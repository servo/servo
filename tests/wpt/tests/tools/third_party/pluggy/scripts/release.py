"""
Release script.
"""

import argparse
from subprocess import check_call
import sys

from colorama import Fore
from colorama import init
from git import Remote
from git import Repo


def create_branch(version):
    """Create a fresh branch from upstream/main"""
    repo = Repo.init(".")
    if repo.is_dirty(untracked_files=True):
        raise RuntimeError("Repository is dirty, please commit/stash your changes.")

    branch_name = f"release-{version}"
    print(f"{Fore.CYAN}Create {branch_name} branch from upstream main")
    upstream = get_upstream(repo)
    upstream.fetch()
    release_branch = repo.create_head(branch_name, upstream.refs.main, force=True)
    release_branch.checkout()
    return repo


def get_upstream(repo: Repo) -> Remote:
    """Find upstream repository for pluggy on the remotes"""
    for remote in repo.remotes:
        for url in remote.urls:
            if url.endswith(("pytest-dev/pluggy.git", "pytest-dev/pluggy")):
                return remote
    raise RuntimeError("could not find pytest-dev/pluggy remote")


def pre_release(version):
    """Generates new docs, release announcements and creates a local tag."""
    create_branch(version)
    changelog(version, write_out=True)

    check_call(["git", "commit", "-a", "-m", f"Preparing release {version}"])

    print()
    print(f"{Fore.GREEN}Please push your branch to your fork and open a PR.")


def changelog(version, write_out=False):
    if write_out:
        addopts = []
    else:
        addopts = ["--draft"]
    print(f"{Fore.CYAN}Generating CHANGELOG")
    check_call(["towncrier", "build", "--yes", "--version", version] + addopts)


def main():
    init(autoreset=True)
    parser = argparse.ArgumentParser()
    parser.add_argument("version", help="Release version")
    options = parser.parse_args()
    try:
        pre_release(options.version)
    except RuntimeError as e:
        print(f"{Fore.RED}ERROR: {e}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
