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

import json
import os
import re
import subprocess
import sys
import textwrap
import xml.etree.ElementTree as ElementTree

from typing import Optional


SUBTEST_RESULT_TRUNCATION = 10


class Item:
    def __init__(self, title: str, body: str, children: list[Item]):
        self.title = title
        self.body = body
        self.children = children

    @classmethod
    def from_result(cls, result: dict, title_key: str = "path", title_prefix: str = "", print_stack=True):
        expected = result["expected"]
        actual = result["actual"]
        title = result[title_key]
        if expected != actual:
            title = f"{actual} [expected {expected}] {title_prefix}{title}"
        else:
            title = f"{actual} {title_prefix}{title}"

        issue_url = "http://github.com/servo/servo/issues/"
        if "issues" in result and result["issues"]:
            issues = ", ".join([f"[#{issue}]({issue_url}{issue})"
                                for issue in result["issues"]])
            title += f" ({issues})"

        stack = result["stack"] if result["stack"] and print_stack else ""
        body = f"{result['message']}\n{stack}".strip()

        subtest_results = result.get("unexpected_subtest_results", [])
        children = [
            cls.from_result(subtest_result, "subtest", "subtest: ", False)
            for subtest_result in subtest_results
        ]
        return cls(title, body, children)

    def to_string(self, bullet: str = "", indent: str = ""):
        output = f"{indent}{bullet}{self.title}\n"
        if self.body:
            output += textwrap.indent(f"{self.body}\n", " " * len(indent + bullet))
        output += "\n".join([child.to_string("• ", indent + "  ")
                             for child in self.children])
        return output.rstrip()

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
                children.append(Item(
                    f"And {len(self.children) - max_children} more unexpected results...",
                    "", []))
            else:
                children = self.children
            container = ElementTree.SubElement(result, "div" if not level else "ul")
            for child in children:
                container.append(child.to_html(level + 1))

        return result


def get_results() -> Optional[Item]:
    unexpected = []
    for filename in sys.argv[1:]:
        with open(filename, encoding="utf-8") as file:
            unexpected += json.load(file)
    unexpected.sort(key=lambda result: result["path"])

    def is_flaky(result):
        return result["flaky"]

    def is_stable_and_known(result):
        return not is_flaky(result) and result["issues"]

    def is_stable_and_unexpected(result):
        return not is_flaky(result) and not result["issues"]

    def add_children(children, results, filter_func, text):
        filtered = [Item.from_result(result) for result in
                    filter(filter_func, results)]
        if filtered:
            children.append(Item(f"{text} ({len(filtered)})", "", filtered))

    children = []
    add_children(children, unexpected, is_flaky, "Flaky unexpected result")
    add_children(children, unexpected, is_stable_and_known,
                 "Stable unexpected results that are known to be intermittent")
    add_children(children, unexpected, is_stable_and_unexpected,
                 "Stable unexpected results")

    run_url = get_github_run_url()
    if run_url:
        text = f"Results from try job ({run_url}):"
    else:
        text = "Results from try job:"
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
    if "head_commit" not in github_context["event"]:
        return None
    commit_title = github_context["event"]["head_commit"]["message"]
    match = re.match(r"^Auto merge of #(\d+)", commit_title)
    return match.group(1) if match else None


def main():
    results = get_results()
    if not results:
        print("Did not find any unexpected results.")
        return

    print(results.to_string())

    pr_number = get_pr_number()
    if pr_number:
        html_string = ElementTree.tostring(results.to_html(), encoding="unicode")
        process = subprocess.Popen(['gh', 'pr', 'comment', pr_number, '-F', '-'], stdin=subprocess.PIPE)
        process.communicate(input=html_string.encode("utf-8"))[0]
    else:
        print("Could not find PR number in environment. Not making GitHub comment.")


if __name__ == "__main__":
    main()
