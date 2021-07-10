"""
This script is part of the pytest release process which is triggered by comments
in issues.

This script is started by the `release-on-comment.yml` workflow, which always executes on
`master` and is triggered by two comment related events:

* https://help.github.com/en/actions/reference/events-that-trigger-workflows#issue-comment-event-issue_comment
* https://help.github.com/en/actions/reference/events-that-trigger-workflows#issues-event-issues

This script receives the payload and a secrets on the command line.

The payload must contain a comment with a phrase matching this pseudo-regular expression:

    @pytestbot please prepare (major )? release from <branch name>

Then the appropriate version will be obtained based on the given branch name:

* a major release from master if "major" appears in the phrase in that position
* a feature or bug fix release from master (based if there are features in the current changelog
  folder)
* a bug fix from a maintenance branch

After that, it will create a release using the `release` tox environment, and push a new PR.

**Secret**: currently the secret is defined in the @pytestbot account, which the core maintainers
have access to. There we created a new secret named `chatops` with write access to the repository.
"""
import argparse
import json
import os
import re
import traceback
from pathlib import Path
from subprocess import CalledProcessError
from subprocess import check_call
from subprocess import check_output
from subprocess import run
from textwrap import dedent
from typing import Dict
from typing import Optional
from typing import Tuple

from colorama import Fore
from colorama import init
from github3.repos import Repository


class InvalidFeatureRelease(Exception):
    pass


SLUG = "pytest-dev/pytest"

PR_BODY = """\
Created automatically from {comment_url}.

Once all builds pass and it has been **approved** by one or more maintainers, the build
can be released by pushing a tag `{version}` to this repository.
"""


def login(token: str) -> Repository:
    import github3

    github = github3.login(token=token)
    owner, repo = SLUG.split("/")
    return github.repository(owner, repo)


def get_comment_data(payload: Dict) -> str:
    if "comment" in payload:
        return payload["comment"]
    else:
        return payload["issue"]


def validate_and_get_issue_comment_payload(
    issue_payload_path: Optional[Path],
) -> Tuple[str, str, bool]:
    payload = json.loads(issue_payload_path.read_text(encoding="UTF-8"))
    body = get_comment_data(payload)["body"]
    m = re.match(r"@pytestbot please prepare (major )?release from ([\w\-_\.]+)", body)
    if m:
        is_major, base_branch = m.group(1) is not None, m.group(2)
    else:
        is_major, base_branch = False, None
    return payload, base_branch, is_major


def print_and_exit(msg) -> None:
    print(msg)
    raise SystemExit(1)


def trigger_release(payload_path: Path, token: str) -> None:
    payload, base_branch, is_major = validate_and_get_issue_comment_payload(
        payload_path
    )
    if base_branch is None:
        url = get_comment_data(payload)["html_url"]
        print_and_exit(
            f"Comment {Fore.CYAN}{url}{Fore.RESET} did not match the trigger command."
        )
    print()
    print(f"Precessing release for branch {Fore.CYAN}{base_branch}")

    repo = login(token)

    issue_number = payload["issue"]["number"]
    issue = repo.issue(issue_number)

    check_call(["git", "checkout", f"origin/{base_branch}"])

    try:
        version = find_next_version(base_branch, is_major)
    except InvalidFeatureRelease as e:
        issue.create_comment(str(e))
        print_and_exit(f"{Fore.RED}{e}")

    error_contents = ""
    try:
        print(f"Version: {Fore.CYAN}{version}")

        release_branch = f"release-{version}"

        run(
            ["git", "config", "user.name", "pytest bot"],
            text=True,
            check=True,
            capture_output=True,
        )
        run(
            ["git", "config", "user.email", "pytestbot@gmail.com"],
            text=True,
            check=True,
            capture_output=True,
        )

        run(
            ["git", "checkout", "-b", release_branch, f"origin/{base_branch}"],
            text=True,
            check=True,
            capture_output=True,
        )

        print(f"Branch {Fore.CYAN}{release_branch}{Fore.RESET} created.")

        # important to use tox here because we have changed branches, so dependencies
        # might have changed as well
        cmdline = ["tox", "-e", "release", "--", version, "--skip-check-links"]
        print("Running", " ".join(cmdline))
        run(
            cmdline, text=True, check=True, capture_output=True,
        )

        oauth_url = f"https://{token}:x-oauth-basic@github.com/{SLUG}.git"
        run(
            ["git", "push", oauth_url, f"HEAD:{release_branch}", "--force"],
            text=True,
            check=True,
            capture_output=True,
        )
        print(f"Branch {Fore.CYAN}{release_branch}{Fore.RESET} pushed.")

        body = PR_BODY.format(
            comment_url=get_comment_data(payload)["html_url"], version=version
        )
        pr = repo.create_pull(
            f"Prepare release {version}",
            base=base_branch,
            head=release_branch,
            body=body,
        )
        print(f"Pull request {Fore.CYAN}{pr.url}{Fore.RESET} created.")

        comment = issue.create_comment(
            f"As requested, opened a PR for release `{version}`: #{pr.number}."
        )
        print(f"Notified in original comment {Fore.CYAN}{comment.url}{Fore.RESET}.")

    except CalledProcessError as e:
        error_contents = f"CalledProcessError\noutput:\n{e.output}\nstderr:\n{e.stderr}"
    except Exception:
        error_contents = f"Exception:\n{traceback.format_exc()}"

    if error_contents:
        link = f"https://github.com/{SLUG}/actions/runs/{os.environ['GITHUB_RUN_ID']}"
        msg = ERROR_COMMENT.format(
            version=version, base_branch=base_branch, contents=error_contents, link=link
        )
        issue.create_comment(msg)
        print_and_exit(f"{Fore.RED}{error_contents}")
    else:
        print(f"{Fore.GREEN}Success.")


ERROR_COMMENT = """\
The request to prepare release `{version}` from {base_branch} failed with:

```
{contents}
```

See: {link}.
"""


def find_next_version(base_branch: str, is_major: bool) -> str:
    output = check_output(["git", "tag"], encoding="UTF-8")
    valid_versions = []
    for v in output.splitlines():
        m = re.match(r"\d.\d.\d+$", v.strip())
        if m:
            valid_versions.append(tuple(int(x) for x in v.split(".")))

    valid_versions.sort()
    last_version = valid_versions[-1]

    changelog = Path("changelog")

    features = list(changelog.glob("*.feature.rst"))
    breaking = list(changelog.glob("*.breaking.rst"))
    is_feature_release = features or breaking

    if is_feature_release and base_branch != "master":
        msg = dedent(
            f"""
            Found features or breaking changes in `{base_branch}`, and feature releases can only be
            created from `master`.":
        """
        )
        msg += "\n".join(f"* `{x.name}`" for x in sorted(features + breaking))
        raise InvalidFeatureRelease(msg)

    if is_major:
        return f"{last_version[0]+1}.0.0"
    elif is_feature_release:
        return f"{last_version[0]}.{last_version[1] + 1}.0"
    else:
        return f"{last_version[0]}.{last_version[1]}.{last_version[2] + 1}"


def main() -> None:
    init(autoreset=True)
    parser = argparse.ArgumentParser()
    parser.add_argument("payload")
    parser.add_argument("token")
    options = parser.parse_args()
    trigger_release(Path(options.payload), options.token)


if __name__ == "__main__":
    main()
