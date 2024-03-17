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
from datetime import datetime

import json
import os
import re
import subprocess
import argparse
import textwrap
import xml.etree.ElementTree as ElementTree

from typing import List, Optional


SUBTEST_RESULT_TRUNCATION = 10


class Item:
    def __init__(self, title: str, body: str, children: list[Item]):
        self.title = title
        self.body = body
        self.children = children

    @classmethod
    def from_result(cls, result: dict, title: Optional[str] = None, print_stack=True):
        expected = result["expected"]
        actual = result["actual"]
        title = title if title else f'`{result["path"]}`'
        if expected != actual:
            title = f"{actual} [expected {expected}] {title}"
        else:
            title = f"{actual} {title}"

        issue_url = "http://github.com/servo/servo/issues/"
        if "issues" in result and result["issues"]:
            issues = ", ".join([f"[#{issue}]({issue_url}{issue})"
                                for issue in result["issues"]])
            title += f" ({issues})"

        stack = result["stack"] if result["stack"] and print_stack else ""
        body = f"{result['message']}\n{stack}".strip()
        if body:
            body = f"\n```\n{body}\n```\n"

        subtest_results = result.get("unexpected_subtest_results", [])
        children = [
            cls.from_result(
                subtest_result,
                f"subtest: `{subtest_result['subtest']}`"
                + (f" \n```\n{subtest_result['message']}\n```\n" if subtest_result['message'] else ""),
                False)
            for subtest_result in subtest_results
        ]
        return cls(title, body, children)

    def to_string(self, bullet: str = "", indent: str = ""):
        output = f"{indent}{bullet}{self.title}\n"
        if self.body:
            output += textwrap.indent(f"{self.body}\n",
                                      " " * len(indent + bullet))
        output += "\n".join([child.to_string("• ", indent + "  ")
                             for child in self.children])
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
            max_children = len(
                self.children) if level < 2 else SUBTEST_RESULT_TRUNCATION
            if len(self.children) > max_children:
                children = self.children[:max_children]
                children.append(Item(
                    f"And {len(self.children) - max_children} more unexpected results...",
                    "", []))
            else:
                children = self.children
            container = ElementTree.SubElement(
                result, "div" if not level else "ul")
            for child in children:
                container.append(child.to_html(level + 1))

        return result


def get_results(filenames: list[str], tag: str = "") -> Optional[Item]:
    unexpected = []
    for filename in filenames:
        try:
            with open(filename, encoding="utf-8") as file:
                unexpected += json.load(file)
        except FileNotFoundError as exception:
            print(exception)
    unexpected.sort(key=lambda result: result["path"])

    def is_flaky(result):
        return result["flaky"]

    def is_stable_and_known(result):
        return not is_flaky(result) and result["issues"]

    def is_stable_and_unexpected(result):
        return not is_flaky(result) and not result["issues"]

    def add_children(children: List[Item], results: List[dict], filter_func, text):
        filtered = [Item.from_result(result) for result in
                    filter(filter_func, results)]
        if filtered:
            children.append(Item(f"{text} ({len(filtered)})", "", filtered))

    children: List[Item] = []
    add_children(children, unexpected, is_flaky, "Flaky unexpected result")
    add_children(children, unexpected, is_stable_and_known,
                 "Stable unexpected results that are known to be intermittent")
    add_children(children, unexpected, is_stable_and_unexpected,
                 "Stable unexpected results")

    run_url = get_github_run_url()
    text = "Test results"
    if tag:
        text += f" for {tag}"
    text += " from try job"
    if run_url:
        text += f" ({run_url})"
    text += ":"
    return Item(text, "", children) if children else None


def get_github_run_url() -> Optional[str]:
    github_context = json.loads(os.environ.get("GITHUB_CONTEXT", "{}"))
    if "repository" not in github_context:
        return None
    if "run_id" not in github_context:
        return None
    repository = github_context['repository']
    run_id = github_context['run_id']
    return f"[#{run_id}](https://github.com/{repository}/actions/runs/{run_id})"


def get_pr_number() -> Optional[str]:
    github_context = json.loads(os.environ.get("GITHUB_CONTEXT", "{}"))
    if "event" not in github_context:
        return None

    # If we have a 'merge_group' in the context, this was triggered by
    # the merge queue.
    if "merge_group" in github_context["event"]:
        commit_title = github_context["event"]["merge_group"]["head_commit"]["message"]
        match = re.match(r"\(#(\d+)\)$", commit_title)
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


def create_check_run(body: str, tag: str = ""):
    # This process is based on the documentation here:
    # https://docs.github.com/en/rest/checks/runs?apiVersion=2022-11-28#create-a-check-runs
    results = json.loads(os.environ.get("RESULTS", "{}"))
    if all(r == 'success' for r in results):
        conclusion = 'success'
    elif "failure" in results:
        conclusion = 'failure'
    elif "cancelled" in results:
        conclusion = 'cancelled'
    else:
        conclusion = 'neutral'

    github_token = os.environ.get("GITHUB_TOKEN")
    github_context = json.loads(os.environ.get("GITHUB_CONTEXT", "{}"))
    if "sha" not in github_context:
        return None
    if "repository" not in github_context:
        return None
    repo = github_context["repository"]
    data = {
        'name': tag,
        'head_sha': github_context["sha"],
        'status': 'completed',
        'started_at': datetime.utcnow().replace(microsecond=0).isoformat() + "Z",
        'conclusion': conclusion,
        'completed_at': datetime.utcnow().replace(microsecond=0).isoformat() + "Z",
        'output': {
            'title': f'Aggregated {tag} report',
            'summary': body,
            'images': [{'alt': 'WPT logo', 'image_url': 'https://avatars.githubusercontent.com/u/37226233'}]
        },
        'actions': [
        ]
    }

    subprocess.Popen(["curl", "-L",
                      "-X", "POST",
                      "-H", "Accept: application/vnd.github+json",
                      "-H", f"Authorization: Bearer {github_token}",
                      "-H", "X-GitHub-Api-Version: 2022-11-28",
                      f"https://api.github.com/repos/{repo}/check-runs",
                      "-d", json.dumps(data)]).wait()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--tag", default="wpt", action="store",
                        help="A string tag used to distinguish the results.")
    args, filenames = parser.parse_known_args()
    results = get_results(filenames, args.tag)
    if not results:
        print("Did not find any unexpected results.")
        create_check_run("Did not find any unexpected results.", args.tag)
        return

    print(results.to_string())

    pr_number = get_pr_number()
    html_string = ElementTree.tostring(
        results.to_html(), encoding="unicode")
    create_check_run(html_string, args.tag)

    if pr_number:
        process = subprocess.Popen(
            ['gh', 'pr', 'comment', pr_number, '-F', '-'], stdin=subprocess.PIPE)
        print(process.communicate(input=html_string.encode("utf-8"))[0])
    else:
        print("Could not find PR number in environment. Not making GitHub comment.")


if __name__ == "__main__":
    main()
