"""
Invoke development tasks.
"""
from pathlib import Path
from subprocess import check_output, check_call

import invoke


@invoke.task(help={"version": "version being released"})
def announce(ctx, version):
    """Generates a new release announcement entry in the docs."""
    # Get our list of authors
    stdout = check_output(["git", "describe", "--abbrev=0", "--tags"])
    stdout = stdout.decode("utf-8")
    last_version = stdout.strip()

    stdout = check_output(
        ["git", "log", "{}..HEAD".format(last_version), "--format=%aN"]
    )
    stdout = stdout.decode("utf-8")

    contributors = set(stdout.splitlines())

    template_name = "release.minor.rst" if version.endswith(
        ".0"
    ) else "release.patch.rst"
    template_text = Path(__file__).parent.joinpath(template_name).read_text(
        encoding="UTF-8"
    )

    contributors_text = "\n".join(
        "* {}".format(name) for name in sorted(contributors)
    ) + "\n"
    text = template_text.format(version=version, contributors=contributors_text)

    target = Path(__file__).parent.joinpath(
        "../doc/en/announce/release-{}.rst".format(version)
    )
    target.write_text(text, encoding="UTF-8")
    print("[generate.announce] Generated {}".format(target.name))

    # Update index with the new release entry
    index_path = Path(__file__).parent.joinpath("../doc/en/announce/index.rst")
    lines = index_path.read_text(encoding="UTF-8").splitlines()
    indent = "   "
    for index, line in enumerate(lines):
        if line.startswith("{}release-".format(indent)):
            new_line = indent + target.stem
            if line != new_line:
                lines.insert(index, new_line)
                index_path.write_text("\n".join(lines) + "\n", encoding="UTF-8")
                print("[generate.announce] Updated {}".format(index_path.name))
            else:
                print(
                    "[generate.announce] Skip {} (already contains release)".format(
                        index_path.name
                    )
                )
            break

    check_call(["git", "add", str(target)])


@invoke.task()
def regen(ctx):
    """Call regendoc tool to update examples and pytest output in the docs."""
    print("[generate.regen] Updating docs")
    check_call(["tox", "-e", "regen"])


@invoke.task()
def make_tag(ctx, version):
    """Create a new, local tag for the release, only if the repository is clean."""
    from git import Repo

    repo = Repo(".")
    if repo.is_dirty():
        print("Current repository is dirty. Please commit any changes and try again.")
        raise invoke.Exit(code=2)

    tag_names = [x.name for x in repo.tags]
    if version in tag_names:
        print("[generate.make_tag] Delete existing tag {}".format(version))
        repo.delete_tag(version)

    print("[generate.make_tag] Create tag {}".format(version))
    repo.create_tag(version)


@invoke.task(help={"version": "version being released"})
def pre_release(ctx, version):
    """Generates new docs, release announcements and creates a local tag."""
    announce(ctx, version)
    regen(ctx)
    changelog(ctx, version, write_out=True)

    msg = "Preparing release version {}".format(version)
    check_call(["git", "commit", "-a", "-m", msg])

    make_tag(ctx, version)

    print()
    print("[generate.pre_release] Please push your branch and open a PR.")


@invoke.task(
    help={
        "version": "version being released",
        "write_out": "write changes to the actual changelog",
    }
)
def changelog(ctx, version, write_out=False):
    if write_out:
        addopts = []
    else:
        addopts = ["--draft"]
    check_call(["towncrier", "--yes", "--version", version] + addopts)
