#!/usr/bin/env python

# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# This allows using types that are defined later in the file.
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import textwrap
import xml.etree.ElementTree as ElementTree
from datetime import datetime, timezone
from typing import Any, Callable, Optional

SUBTEST_RESULT_TRUNCATION = 10

TestResult = dict[str, Any]


class Item:
    def __init__(self, title: str, body: str, children: list[Item]):
        self.title = title
        self.body = body
        self.children = children

    @classmethod
    def from_result(cls, result: TestResult, title: Optional[str] = None, print_stack: bool = True):
        expected = result["expected"]
        actual = result["actual"]
        title = title if title else f"`{result['path']}`"
        if expected != actual:
            title = f"{actual} [expected {expected}] {title}"
        else:
            title = f"{actual} {title}"

        issue_url = "http://github.com/servo/servo/issues/"
        if "issues" in result and result["issues"]:
            issues = ", ".join([f"[#{issue}]({issue_url}{issue})" for issue in result["issues"]])
            title += f" ({issues})"

        stack = result["stack"] if result["stack"] and print_stack else ""
        body = f"{result['message']}\n{stack}".strip()
        if body:
            body = f"\n```\n{body}\n```\n"

        subtest_results = result.get("unexpected_subtest_results", [])
        children = [
            Item.from_result(
                subtest_result,
                f"subtest: `{subtest_result['subtest']}`"
                + (f" \n```\n{subtest_result['message']}\n```\n" if subtest_result["message"] else ""),
                False,
            )
            for subtest_result in subtest_results
        ]
        return cls(title, body, children)

    def to_string(self, bullet: str = "", indent: str = "") -> str:
        output = f"{indent}{bullet}{self.title}\n"
        if self.body:
            output += textwrap.indent(f"{self.body}\n", " " * len(indent + bullet))
        output += "\n".join([child.to_string("• ", indent + "  ") for child in self.children])
        return output.rstrip().replace("`", "")

    def to_html(self, level: int = 0) -> ElementTree.Element:
        if level == 0:
            title = result = ElementTree.Element("span")
        elif level == 1:
            result = ElementTree.Element("details")
            title = ElementTree.SubElement(result, "summary")
        else:
            result = ElementTree.Element("li")
            title = ElementTree.SubElement(result, "span")
        title.text = self.title

        if self.children:
            # Some tests have dozens of failing tests, which overwhelm the
            # output. Limit the output for subtests in GitHub comment output.
            max_children = len(self.children) if level < 2 else SUBTEST_RESULT_TRUNCATION
            if len(self.children) > max_children:
                children = self.children[:max_children]
                children.append(Item(f"And {len(self.children) - max_children} more unexpected results...", "", []))
            else:
                children = self.children
            container = ElementTree.SubElement(result, "div" if not level else "ul")
            for child in children:
                container.append(child.to_html(level + 1))

        return result


def is_flaky(result: TestResult) -> bool:
    return result["flaky"]


def is_stable_and_known(result: TestResult) -> bool:
    return not is_flaky(result) and result["issues"]


def is_stable_and_unexpected(result: TestResult) -> bool:
    return not is_flaky(result) and not result["issues"]


def get_results(unexpected: list[TestResult], tag: str = "") -> Optional[Item]:
    def add_children(
        children: list[Item], results: list[TestResult], filter_func: Callable[[TestResult], bool], text: str
    ) -> None:
        filtered = [Item.from_result(result) for result in filter(filter_func, results)]
        if filtered:
            children.append(Item(f"{text} ({len(filtered)})", "", filtered))

    children: list[Item] = []
    add_children(children, unexpected, is_flaky, "Flaky unexpected result")
    add_children(
        children, unexpected, is_stable_and_known, "Stable unexpected results that are known to be intermittent"
    )
    add_children(children, unexpected, is_stable_and_unexpected, "Stable unexpected results")

    run_url = get_github_run_url()
    text = "Test results"
    if tag:
        text += f" for {tag}"

    job_type = "merge queue job" if is_merge_queue_job() else "try job"
    text += f" from {job_type}"

    if run_url:
        text += f" ({run_url})"
    text += ":"
    return Item(text, "", children) if children else None


def get_results_per_subsuite(filenames: list[str], tag: str = "") -> list[tuple[Item, str, bool]]:
    """Returns the Item, tag, and whether the subsuite had any stable unexpected results for each
    subsuite."""

    unexpected: list[TestResult] = []
    for filename in filenames:
        try:
            with open(filename, encoding="utf-8") as file:
                unexpected += json.load(file)
        except FileNotFoundError as exception:
            print(exception)
    unexpected.sort(key=lambda result: result["path"])
    subsuites = set(result.get("subsuite", "") for result in unexpected)
    results: list[tuple[Item, str, bool]] = []
    for subsuite in subsuites:
        subsuite_tag = f"{tag} {subsuite}" if subsuite else tag
        subsuite_unexpected = list(filter(lambda result: result.get("subsuite", "") == subsuite, unexpected))
        subsuite_results = get_results(subsuite_unexpected, subsuite_tag)
        had_stable_unexpected_results = any(is_stable_and_unexpected(result) for result in subsuite_unexpected)
        if subsuite_results:
            results.append((subsuite_results, subsuite_tag, had_stable_unexpected_results))
    return results


def get_github_run_url() -> Optional[str]:
    github_context = json.loads(os.environ.get("GITHUB_CONTEXT", "{}"))
    if "repository" not in github_context:
        return None
    if "run_id" not in github_context:
        return None
    repository = github_context["repository"]
    run_id = github_context["run_id"]
    return f"[#{run_id}](https://github.com/{repository}/actions/runs/{run_id})"


def is_merge_queue_job(github_context: Optional[dict[str, Any]] = None) -> bool:
    if github_context is None:
        github_context = json.loads(os.environ.get("GITHUB_CONTEXT", "{}"))
        assert github_context is not None
    return github_context.get("event_name") == "merge_group"


def get_pr_number() -> Optional[str]:
    github_context = json.loads(os.environ.get("GITHUB_CONTEXT", "{}"))
    if "event" not in github_context:
        return None

    # If we have a 'merge_group' in the context, this was triggered by
    # the merge queue.
    if is_merge_queue_job(github_context):
        message = github_context["event"]["merge_group"]["head_commit"]["message"]
        if not message:
            return None
        commit_title = message.splitlines()[0]
        match = re.search(r"\(#(\d+)\)$", commit_title)
        return match.group(1) if match else None

    # If we have an 'issue' in the context, this was triggered by a try comment
    # on a PR.
    if "issue" in github_context["event"]:
        return str(github_context["event"]["issue"]["number"])

    # If we have an 'number' in the context, this was triggered by "pull_request" or
    # "pull_request_target" event.
    if "number" in github_context["event"]:
        return str(github_context["event"]["number"])

    return None


def get_commit_sha() -> str:
    return subprocess.check_output(["git", "log", "-1", "--format=%H"]).decode("utf-8").strip()


def create_github_reports(body: str, tag: str = "") -> None:
    # GitHub will set a special environment variable which points to a file where
    # a job summary can be appended. This is produced in addition to the GitHub
    # check run below, as it has a larger length limit.
    if "GITHUB_STEP_SUMMARY" in os.environ:
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as f:
            print(body, file=f)

    # Create a GitHub check run. This is more user friendly than a job summary,
    # but has a very limited size -- thus it may fail.
    # This process is based on the documentation here:
    # https://docs.github.com/en/rest/checks/runs?apiVersion=2022-11-28#create-a-check-runs
    results = json.loads(os.environ.get("RESULTS", "{}"))
    if all(r == "success" for r in results):
        conclusion = "success"
    elif "failure" in results:
        conclusion = "failure"
    elif "cancelled" in results:
        conclusion = "cancelled"
    else:
        conclusion = "neutral"

    github_token = os.environ.get("GITHUB_TOKEN")
    github_context = json.loads(os.environ.get("GITHUB_CONTEXT", "{}"))
    if "repository" not in github_context:
        return None
    repo = github_context["repository"]
    data = {
        "name": tag,
        "head_sha": get_commit_sha(),
        "status": "completed",
        "started_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "conclusion": conclusion,
        "completed_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "output": {
            "title": f"Aggregated {tag} report",
            "summary": body,
            "images": [{"alt": "WPT logo", "image_url": "https://avatars.githubusercontent.com/u/37226233"}],
        },
        "actions": [],
    }

    subprocess.Popen(
        [
            "curl",
            "-L",
            "-X",
            "POST",
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            f"Authorization: Bearer {github_token}",
            "-H",
            "X-GitHub-Api-Version: 2022-11-28",
            f"https://api.github.com/repos/{repo}/check-runs",
            "-d",
            json.dumps(data),
        ]
    ).wait()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tag", default="wpt", action="store", help="A string tag used to distinguish the results.")
    args, filenames = parser.parse_known_args()
    results = get_results_per_subsuite(filenames, args.tag)
    if not results:
        print("Did not find any unexpected results.")
        create_github_reports("Did not find any unexpected results.", args.tag)
        return

    for subsuite_results, subsuite_tag, had_stable_unexpected in results:
        print(subsuite_results.to_string())

        html_string = ElementTree.tostring(subsuite_results.to_html(), encoding="unicode")
        create_github_reports(html_string, subsuite_tag)

        # Don't make a GitHub comment for a merge queue without stable unexpected results.
        if not had_stable_unexpected and is_merge_queue_job():
            continue

        pr_number = get_pr_number()
        if pr_number:
            process = subprocess.Popen(
                ["gh", "pr", "comment", pr_number, "-F", "-"],
                stdin=subprocess.PIPE,
                text=True,
            )
            print(process.communicate(input=html_string)[0])
        else:
            print("Could not find PR number in environment. Not making GitHub comment.")


if __name__ == "__main__":
    main()
