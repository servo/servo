# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# pylint: disable=missing-docstring

"""This modules contains some abstractions of GitHub repositories. It could one
day be entirely replaced with something like PyGithub."""

# This allows using types that are defined later in the file.
from __future__ import annotations

import logging
import urllib

from typing import Optional, TYPE_CHECKING

import requests

if TYPE_CHECKING:
    from . import WPTSync

USER_AGENT = "Servo web-platform-test sync service"
TIMEOUT = 30  # 30 seconds


def authenticated(sync: WPTSync, method, url, json=None) -> requests.Response:
    logging.info("  → Request: %s %s", method, url)
    if json:
        logging.info("  → Request JSON: %s", json)

    headers = {
        "Authorization": f"Bearer {sync.github_api_token}",
        "User-Agent": USER_AGENT,
    }

    url = urllib.parse.urljoin(sync.github_api_url, url)
    response = requests.request(
        method, url, headers=headers, json=json, timeout=TIMEOUT
    )
    if int(response.status_code / 100) != 2:
        raise ValueError(
            f"Got unexpected {response.status_code} response: {response.text}"
        )
    return response


class GithubRepository:
    """
    This class allows interacting with a single GitHub repository.
    """

    def __init__(self, sync: WPTSync, repo: str, default_branch: str):
        self.sync = sync
        self.repo = repo
        self.default_branch = default_branch
        self.org = repo.split("/")[0]
        self.pulls_url = f"repos/{self.repo}/pulls"

    def __str__(self):
        return self.repo

    def get_pull_request(self, number: int) -> PullRequest:
        return PullRequest(self, number)

    def get_branch(self, name: str) -> GithubBranch:
        return GithubBranch(self, name)

    def get_open_pull_request_for_branch(
        self, branch: GithubBranch
    ) -> Optional[PullRequest]:
        """If this repository has an open pull request with the
        given source head reference targeting the main branch,
        return the first matching pull request, otherwise return None."""

        params = "+".join([
            "is:pr",
            "state:open",
            f"repo:{self.repo}",
            f"author:{branch.repo.org}",
            f"head:{branch.name}",
        ])
        response = authenticated(self.sync, "GET", f"search/issues?q={params}")
        if int(response.status_code / 100) != 2:
            return None

        json = response.json()
        if not isinstance(json, dict) or \
           "total_count" not in json or \
           "items" not in json:
            raise ValueError(
                f"Got unexpected response from GitHub search: {response.text}"
            )

        if json["total_count"] < 1:
            return None

        return self.get_pull_request(json["items"][0]["number"])

    def open_pull_request(self, branch: GithubBranch, title: str, body: str):
        data = {
            "title": title,
            "head": branch.get_pr_head_reference_for_repo(self),
            "base": self.default_branch,
            "body": body,
            "maintainer_can_modify": False,
        }
        response = authenticated(self.sync, "POST", self.pulls_url, json=data)
        return self.get_pull_request(response.json()["number"])


class GithubBranch:
    def __init__(self, repo: GithubRepository, branch_name: str):
        self.repo = repo
        self.name = branch_name

    def __str__(self):
        return f"{self.repo}/{self.name}"

    def get_pr_head_reference_for_repo(self, other_repo: GithubRepository) -> str:
        """Get the head reference to use in pull requests for the given repository.
        If the organization is the same this is just `<branch>` otherwise
        it will be `<org>:<branch>`."""
        if self.repo.org == other_repo.org:
            return self.name
        return f"{self.repo.org}:{self.name}"


class PullRequest:
    """
    This class allows interacting with a single pull request on GitHub.
    """

    def __init__(self, repo: GithubRepository, number: int):
        self.repo = repo
        self.context = repo.sync
        self.number = number
        self.base_url = f"repos/{self.repo.repo}/pulls/{self.number}"
        self.base_issues_url = f"repos/{self.repo.repo}/issues/{self.number}"

    def __str__(self):
        return f"{self.repo}#{self.number}"

    def api(self, *args, **kwargs) -> requests.Response:
        return authenticated(self.context, *args, **kwargs)

    def leave_comment(self, comment: str):
        return self.api(
            "POST", f"{self.base_issues_url}/comments", json={"body": comment}
        )

    def change(
        self,
        state: Optional[str] = None,
        title: Optional[str] = None,
        body: Optional[str] = None,
    ):
        data = {}
        if title:
            data["title"] = title
        if body:
            data["body"] = body
        if state:
            data["state"] = state
        return self.api("PATCH", self.base_url, json=data)

    def remove_label(self, label: str):
        self.api("DELETE", f"{self.base_issues_url}/labels/{label}")

    def add_labels(self, labels: list[str]):
        self.api("POST", f"{self.base_issues_url}/labels", json=labels)

    def merge(self):
        self.api("PUT", f"{self.base_url}/merge", json={"merge_method": "rebase"})
