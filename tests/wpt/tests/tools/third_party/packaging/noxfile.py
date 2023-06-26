# mypy: disallow-untyped-defs=False, disallow-untyped-calls=False

import contextlib
import datetime
import difflib
import glob
import os
import re
import shutil
import subprocess
import sys
import tempfile
import textwrap
import time
import webbrowser
from pathlib import Path

import nox

nox.options.sessions = ["lint"]
nox.options.reuse_existing_virtualenvs = True


@nox.session(python=["3.6", "3.7", "3.8", "3.9", "3.10", "pypy3"])
def tests(session):
    def coverage(*args):
        session.run("python", "-m", "coverage", *args)

    # Once coverage 5 is used then `.coverage` can move into `pyproject.toml`.
    session.install("coverage<5.0.0", "pretend", "pytest>=6.2.0", "pip>=9.0.2")
    session.install(".")

    if "pypy" not in session.python:
        coverage(
            "run",
            "--source",
            "packaging/",
            "-m",
            "pytest",
            "--strict-markers",
            *session.posargs,
        )
        coverage("report", "-m", "--fail-under", "100")
    else:
        # Don't do coverage tracking for PyPy, since it's SLOW.
        session.run(
            "python",
            "-m",
            "pytest",
            "--capture=no",
            "--strict-markers",
            *session.posargs,
        )


@nox.session(python="3.9")
def lint(session):
    # Run the linters (via pre-commit)
    session.install("pre-commit")
    session.run("pre-commit", "run", "--all-files")

    # Check the distribution
    session.install("build", "twine")
    session.run("pyproject-build")
    session.run("twine", "check", *glob.glob("dist/*"))


@nox.session(python="3.9")
def docs(session):
    shutil.rmtree("docs/_build", ignore_errors=True)
    session.install("furo")
    session.install("-e", ".")

    variants = [
        # (builder, dest)
        ("html", "html"),
        ("latex", "latex"),
        ("doctest", "html"),
    ]

    for builder, dest in variants:
        session.run(
            "sphinx-build",
            "-W",
            "-b",
            builder,
            "-d",
            "docs/_build/doctrees/" + dest,
            "docs",  # source directory
            "docs/_build/" + dest,  # output directory
        )


@nox.session
def release(session):
    package_name = "packaging"
    version_file = Path(f"{package_name}/__about__.py")
    changelog_file = Path("CHANGELOG.rst")

    try:
        release_version = _get_version_from_arguments(session.posargs)
    except ValueError as e:
        session.error(f"Invalid arguments: {e}")
        return

    # Check state of working directory and git.
    _check_working_directory_state(session)
    _check_git_state(session, release_version)

    # Prepare for release.
    _changelog_update_unreleased_title(release_version, file=changelog_file)
    session.run("git", "add", str(changelog_file), external=True)
    _bump(session, version=release_version, file=version_file, kind="release")

    # Tag the release commit.
    # fmt: off
    session.run(
        "git", "tag",
        "-s", release_version,
        "-m", f"Release {release_version}",
        external=True,
    )
    # fmt: on

    # Prepare for development.
    _changelog_add_unreleased_title(file=changelog_file)
    session.run("git", "add", str(changelog_file), external=True)

    major, minor = map(int, release_version.split("."))
    next_version = f"{major}.{minor + 1}.dev0"
    _bump(session, version=next_version, file=version_file, kind="development")

    # Checkout the git tag.
    session.run("git", "checkout", "-q", release_version, external=True)

    session.install("build", "twine")

    # Build the distribution.
    session.run("python", "-m", "build")

    # Check what files are in dist/ for upload.
    files = sorted(glob.glob("dist/*"))
    expected = [
        f"dist/{package_name}-{release_version}-py3-none-any.whl",
        f"dist/{package_name}-{release_version}.tar.gz",
    ]
    if files != expected:
        diff_generator = difflib.context_diff(
            expected, files, fromfile="expected", tofile="got", lineterm=""
        )
        diff = "\n".join(diff_generator)
        session.error(f"Got the wrong files:\n{diff}")

    # Get back out into main.
    session.run("git", "checkout", "-q", "main", external=True)

    # Check and upload distribution files.
    session.run("twine", "check", *files)

    # Push the commits and tag.
    # NOTE: The following fails if pushing to the branch is not allowed. This can
    #       happen on GitHub, if the main branch is protected, there are required
    #       CI checks and "Include administrators" is enabled on the protection.
    session.run("git", "push", "upstream", "main", release_version, external=True)

    # Upload the distribution.
    session.run("twine", "upload", *files)

    # Open up the GitHub release page.
    webbrowser.open("https://github.com/pypa/packaging/releases")


# -----------------------------------------------------------------------------
# Helpers
# -----------------------------------------------------------------------------
def _get_version_from_arguments(arguments):
    """Checks the arguments passed to `nox -s release`.

    Only 1 argument that looks like a version? Return the argument.
    Otherwise, raise a ValueError describing what's wrong.
    """
    if len(arguments) != 1:
        raise ValueError("Expected exactly 1 argument")

    version = arguments[0]
    parts = version.split(".")

    if len(parts) != 2:
        # Not of the form: YY.N
        raise ValueError("not of the form: YY.N")

    if not all(part.isdigit() for part in parts):
        # Not all segments are integers.
        raise ValueError("non-integer segments")

    # All is good.
    return version


def _check_working_directory_state(session):
    """Check state of the working directory, prior to making the release."""
    should_not_exist = ["build/", "dist/"]

    bad_existing_paths = list(filter(os.path.exists, should_not_exist))
    if bad_existing_paths:
        session.error(f"Remove {', '.join(bad_existing_paths)} and try again")


def _check_git_state(session, version_tag):
    """Check state of the git repository, prior to making the release."""
    # Ensure the upstream remote pushes to the correct URL.
    allowed_upstreams = [
        "git@github.com:pypa/packaging.git",
        "https://github.com/pypa/packaging.git",
    ]
    result = subprocess.run(
        ["git", "remote", "get-url", "--push", "upstream"],
        capture_output=True,
        encoding="utf-8",
    )
    if result.stdout.rstrip() not in allowed_upstreams:
        session.error(f"git remote `upstream` is not one of {allowed_upstreams}")
    # Ensure we're on main branch for cutting a release.
    result = subprocess.run(
        ["git", "rev-parse", "--abbrev-ref", "HEAD"],
        capture_output=True,
        encoding="utf-8",
    )
    if result.stdout != "main\n":
        session.error(f"Not on main branch: {result.stdout!r}")

    # Ensure there are no uncommitted changes.
    result = subprocess.run(
        ["git", "status", "--porcelain"], capture_output=True, encoding="utf-8"
    )
    if result.stdout:
        print(result.stdout, end="", file=sys.stderr)
        session.error("The working tree has uncommitted changes")

    # Ensure this tag doesn't exist already.
    result = subprocess.run(
        ["git", "rev-parse", version_tag], capture_output=True, encoding="utf-8"
    )
    if not result.returncode:
        session.error(f"Tag already exists! {version_tag} -- {result.stdout!r}")

    # Back up the current git reference, in a tag that's easy to clean up.
    _release_backup_tag = "auto/release-start-" + str(int(time.time()))
    session.run("git", "tag", _release_backup_tag, external=True)


def _bump(session, *, version, file, kind):
    session.log(f"Bump version to {version!r}")
    contents = file.read_text()
    new_contents = re.sub(
        '__version__ = "(.+)"', f'__version__ = "{version}"', contents
    )
    file.write_text(new_contents)

    session.log("git commit")
    subprocess.run(["git", "add", str(file)])
    subprocess.run(["git", "commit", "-m", f"Bump for {kind}"])


@contextlib.contextmanager
def _replace_file(original_path):
    # Create a temporary file.
    fh, replacement_path = tempfile.mkstemp()

    try:
        with os.fdopen(fh, "w") as replacement:
            with open(original_path) as original:
                yield original, replacement
    except Exception:
        raise
    else:
        shutil.copymode(original_path, replacement_path)
        os.remove(original_path)
        shutil.move(replacement_path, original_path)


def _changelog_update_unreleased_title(version, *, file):
    """Update an "*unreleased*" heading to "{version} - {date}" """
    yyyy_mm_dd = datetime.datetime.today().strftime("%Y-%m-%d")
    title = f"{version} - {yyyy_mm_dd}"

    with _replace_file(file) as (original, replacement):
        for line in original:
            if line == "*unreleased*\n":
                replacement.write(f"{title}\n")
                replacement.write(len(title) * "~" + "\n")
                # Skip processing the next line (the heading underline for *unreleased*)
                # since we already wrote the heading underline.
                next(original)
            else:
                replacement.write(line)


def _changelog_add_unreleased_title(*, file):
    with _replace_file(file) as (original, replacement):
        # Duplicate first 3 lines from the original file.
        for _ in range(3):
            line = next(original)
            replacement.write(line)

        # Write the heading.
        replacement.write(
            textwrap.dedent(
                """\
                *unreleased*
                ~~~~~~~~~~~~

                No unreleased changes.

                """
            )
        )

        # Duplicate all the remaining lines.
        for line in original:
            replacement.write(line)
