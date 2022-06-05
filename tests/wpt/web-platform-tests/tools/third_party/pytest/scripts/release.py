"""Invoke development tasks."""
import argparse
import os
from pathlib import Path
from subprocess import call
from subprocess import check_call
from subprocess import check_output

from colorama import Fore
from colorama import init


def announce(version, template_name, doc_version):
    """Generates a new release announcement entry in the docs."""
    # Get our list of authors
    stdout = check_output(["git", "describe", "--abbrev=0", "--tags"])
    stdout = stdout.decode("utf-8")
    last_version = stdout.strip()

    stdout = check_output(["git", "log", f"{last_version}..HEAD", "--format=%aN"])
    stdout = stdout.decode("utf-8")

    contributors = {
        name
        for name in stdout.splitlines()
        if not name.endswith("[bot]") and name != "pytest bot"
    }

    template_text = (
        Path(__file__).parent.joinpath(template_name).read_text(encoding="UTF-8")
    )

    contributors_text = "\n".join(f"* {name}" for name in sorted(contributors)) + "\n"
    text = template_text.format(
        version=version, contributors=contributors_text, doc_version=doc_version
    )

    target = Path(__file__).parent.joinpath(f"../doc/en/announce/release-{version}.rst")
    target.write_text(text, encoding="UTF-8")
    print(f"{Fore.CYAN}[generate.announce] {Fore.RESET}Generated {target.name}")

    # Update index with the new release entry
    index_path = Path(__file__).parent.joinpath("../doc/en/announce/index.rst")
    lines = index_path.read_text(encoding="UTF-8").splitlines()
    indent = "   "
    for index, line in enumerate(lines):
        if line.startswith(f"{indent}release-"):
            new_line = indent + target.stem
            if line != new_line:
                lines.insert(index, new_line)
                index_path.write_text("\n".join(lines) + "\n", encoding="UTF-8")
                print(
                    f"{Fore.CYAN}[generate.announce] {Fore.RESET}Updated {index_path.name}"
                )
            else:
                print(
                    f"{Fore.CYAN}[generate.announce] {Fore.RESET}Skip {index_path.name} (already contains release)"
                )
            break

    check_call(["git", "add", str(target)])


def regen(version):
    """Call regendoc tool to update examples and pytest output in the docs."""
    print(f"{Fore.CYAN}[generate.regen] {Fore.RESET}Updating docs")
    check_call(
        ["tox", "-e", "regen"],
        env={**os.environ, "SETUPTOOLS_SCM_PRETEND_VERSION_FOR_PYTEST": version},
    )


def fix_formatting():
    """Runs pre-commit in all files to ensure they are formatted correctly"""
    print(
        f"{Fore.CYAN}[generate.fix linting] {Fore.RESET}Fixing formatting using pre-commit"
    )
    call(["pre-commit", "run", "--all-files"])


def check_links():
    """Runs sphinx-build to check links"""
    print(f"{Fore.CYAN}[generate.check_links] {Fore.RESET}Checking links")
    check_call(["tox", "-e", "docs-checklinks"])


def pre_release(version, template_name, doc_version, *, skip_check_links):
    """Generates new docs, release announcements and creates a local tag."""
    announce(version, template_name, doc_version)
    regen(version)
    changelog(version, write_out=True)
    fix_formatting()
    if not skip_check_links:
        check_links()

    msg = f"Prepare release version {version}"
    check_call(["git", "commit", "-a", "-m", msg])

    print()
    print(f"{Fore.CYAN}[generate.pre_release] {Fore.GREEN}All done!")
    print()
    print("Please push your branch and open a PR.")


def changelog(version, write_out=False):
    addopts = [] if write_out else ["--draft"]
    check_call(["towncrier", "--yes", "--version", version] + addopts)


def main():
    init(autoreset=True)
    parser = argparse.ArgumentParser()
    parser.add_argument("version", help="Release version")
    parser.add_argument(
        "template_name", help="Name of template file to use for release announcement"
    )
    parser.add_argument(
        "doc_version", help="For prereleases, the version to link to in the docs"
    )
    parser.add_argument("--skip-check-links", action="store_true", default=False)
    options = parser.parse_args()
    pre_release(
        options.version,
        options.template_name,
        options.doc_version,
        skip_check_links=options.skip_check_links,
    )


if __name__ == "__main__":
    main()
