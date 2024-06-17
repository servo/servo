# mypy: disallow-untyped-defs
"""
This script is part of the pytest release process which is triggered manually in the Actions
tab of the repository.

The user will need to enter the base branch to start the release from (for example
``6.1.x`` or ``main``) and if it should be a major release.

The appropriate version will be obtained based on the given branch automatically.

After that, it will create a release using the `release` tox environment, and push a new PR.

**Token**: currently the token from the GitHub Actions is used, pushed with
`pytest bot <pytestbot@gmail.com>` commit author.
"""

import argparse
from pathlib import Path
import re
from subprocess import check_call
from subprocess import check_output
from subprocess import run

from colorama import Fore
from colorama import init
from github3.repos import Repository


class InvalidFeatureRelease(Exception):
    pass


SLUG = "pytest-dev/pytest"

PR_BODY = """\
Created by the [prepare release pr]\
(https://github.com/pytest-dev/pytest/actions/workflows/prepare-release-pr.yml) workflow.

Once all builds pass and it has been **approved** by one or more maintainers, start the \
[deploy](https://github.com/pytest-dev/pytest/actions/workflows/deploy.yml) workflow, using these parameters:

* `Use workflow from`: `release-{version}`.
* `Release version`: `{version}`.

Or execute on the command line:

```console
gh workflow run deploy.yml -r release-{version} -f version={version}
```

After the workflow has been approved by a core maintainer, the package will be uploaded to PyPI automatically.
"""


def login(token: str) -> Repository:
    import github3

    github = github3.login(token=token)
    owner, repo = SLUG.split("/")
    return github.repository(owner, repo)


def prepare_release_pr(
    base_branch: str, is_major: bool, token: str, prerelease: str
) -> None:
    print()
    print(f"Processing release for branch {Fore.CYAN}{base_branch}")

    check_call(["git", "checkout", f"origin/{base_branch}"])

    changelog = Path("changelog")

    features = list(changelog.glob("*.feature.rst"))
    breaking = list(changelog.glob("*.breaking.rst"))
    is_feature_release = bool(features or breaking)

    try:
        version = find_next_version(
            base_branch, is_major, is_feature_release, prerelease
        )
    except InvalidFeatureRelease as e:
        print(f"{Fore.RED}{e}")
        raise SystemExit(1) from None

    print(f"Version: {Fore.CYAN}{version}")

    release_branch = f"release-{version}"

    run(
        ["git", "config", "user.name", "pytest bot"],
        check=True,
    )
    run(
        ["git", "config", "user.email", "pytestbot@gmail.com"],
        check=True,
    )

    run(
        ["git", "checkout", "-b", release_branch, f"origin/{base_branch}"],
        check=True,
    )

    print(f"Branch {Fore.CYAN}{release_branch}{Fore.RESET} created.")

    if is_major:
        template_name = "release.major.rst"
    elif prerelease:
        template_name = "release.pre.rst"
    elif is_feature_release:
        template_name = "release.minor.rst"
    else:
        template_name = "release.patch.rst"

    # important to use tox here because we have changed branches, so dependencies
    # might have changed as well
    cmdline = [
        "tox",
        "-e",
        "release",
        "--",
        version,
        template_name,
        release_branch,  # doc_version
        "--skip-check-links",
    ]
    print("Running", " ".join(cmdline))
    run(
        cmdline,
        check=True,
    )

    oauth_url = f"https://{token}:x-oauth-basic@github.com/{SLUG}.git"
    run(
        ["git", "push", oauth_url, f"HEAD:{release_branch}", "--force"],
        check=True,
    )
    print(f"Branch {Fore.CYAN}{release_branch}{Fore.RESET} pushed.")

    body = PR_BODY.format(version=version)
    repo = login(token)
    pr = repo.create_pull(
        f"Prepare release {version}",
        base=base_branch,
        head=release_branch,
        body=body,
    )
    print(f"Pull request {Fore.CYAN}{pr.url}{Fore.RESET} created.")


def find_next_version(
    base_branch: str, is_major: bool, is_feature_release: bool, prerelease: str
) -> str:
    output = check_output(["git", "tag"], encoding="UTF-8")
    valid_versions = []
    for v in output.splitlines():
        m = re.match(r"\d.\d.\d+$", v.strip())
        if m:
            valid_versions.append(tuple(int(x) for x in v.split(".")))

    valid_versions.sort()
    last_version = valid_versions[-1]

    if is_major:
        return f"{last_version[0]+1}.0.0{prerelease}"
    elif is_feature_release:
        return f"{last_version[0]}.{last_version[1] + 1}.0{prerelease}"
    else:
        return f"{last_version[0]}.{last_version[1]}.{last_version[2] + 1}{prerelease}"


def main() -> None:
    init(autoreset=True)
    parser = argparse.ArgumentParser()
    parser.add_argument("base_branch")
    parser.add_argument("token")
    parser.add_argument("--major", action="store_true", default=False)
    parser.add_argument("--prerelease", default="")
    options = parser.parse_args()
    prepare_release_pr(
        base_branch=options.base_branch,
        is_major=options.major,
        token=options.token,
        prerelease=options.prerelease,
    )


if __name__ == "__main__":
    main()
