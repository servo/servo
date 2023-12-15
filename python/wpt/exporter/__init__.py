# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# pylint: disable=broad-except
# pylint: disable=dangerous-default-value
# pylint: disable=fixme
# pylint: disable=missing-docstring

# This allows using types that are defined later in the file.
from __future__ import annotations

import dataclasses
import json
import logging
import re
import shutil
import subprocess

from typing import Callable, Optional

from .common import \
    CLOSING_EXISTING_UPSTREAM_PR, \
    NO_SYNC_SIGNAL, \
    NO_UPSTREAMBLE_CHANGES_COMMENT, \
    OPENED_NEW_UPSTREAM_PR, \
    UPDATED_EXISTING_UPSTREAM_PR, \
    UPDATED_TITLE_IN_EXISTING_UPSTREAM_PR, \
    UPSTREAMABLE_PATH, \
    wpt_branch_name_from_servo_pr_number

from .github import GithubRepository, PullRequest
from .step import \
    AsyncValue, \
    ChangePRStep, \
    CommentStep, \
    CreateOrUpdateBranchForPRStep, \
    MergePRStep, \
    OpenPRStep, \
    RemoveBranchForPRStep, \
    Step


class LocalGitRepo:
    def __init__(self, path: str, sync: WPTSync):
        self.path = path
        self.sync = sync

        # We pass env to subprocess, which may clobber PATH, so we need to find
        # git in advance and run the subprocess by its absolute path.
        self.git_path = shutil.which("git")

    def run_without_encoding(self, *args, env: dict = {}):
        command_line = [self.git_path] + list(args)
        logging.info("  → Execution (cwd='%s'): %s",
                     self.path, " ".join(command_line))

        env.setdefault("GIT_AUTHOR_EMAIL", self.sync.github_email)
        env.setdefault("GIT_COMMITTER_EMAIL", self.sync.github_email)
        env.setdefault("GIT_AUTHOR_NAME", self.sync.github_name)
        env.setdefault("GIT_COMMITTER_NAME", self.sync.github_name)

        try:
            return subprocess.check_output(
                command_line, cwd=self.path, env=env, stderr=subprocess.STDOUT
            )
        except subprocess.CalledProcessError as exception:
            logging.warning("Process execution failed with output:\n%s",
                            exception.output.decode("utf-8", errors="surrogateescape"))
            raise exception

    def run(self, *args, env: dict = {}):
        return (
            self
            .run_without_encoding(*args, env=env)
            .decode("utf-8", errors="surrogateescape")
        )


@dataclasses.dataclass()
class SyncRun:
    sync: WPTSync
    servo_pr: PullRequest
    upstream_pr: AsyncValue[PullRequest]
    step_callback: Optional[Callable[[Step], None]]
    steps: list[Step] = dataclasses.field(default_factory=list)

    def make_comment(self, template: str) -> str:
        upstream_pr = self.upstream_pr.value() if self.upstream_pr.has_value() else ""
        return template.format(
            upstream_pr=upstream_pr,
            servo_pr=self.servo_pr,
        )

    def add_step(self, step) -> Optional[AsyncValue]:
        self.steps.append(step)
        return step.provides()

    def run(self):
        # This loop always removes the first step and runs it, because
        # individual steps can modify the list of steps. For instance, if a
        # step fails, it might clear the remaining steps and replace them with
        # steps that report the error to GitHub.
        while self.steps:
            step = self.steps.pop(0)
            step.run(self)
            if self.step_callback:
                self.step_callback(step)

    @staticmethod
    def clean_up_body_text(body: str) -> str:
        # Turn all bare or relative issue references into unlinked ones, so that
        # the PR doesn't inadvertently close or link to issues in the upstream
        # repository.
        return (
            re.sub(
                r"(^|\s)(\w*)#([1-9]\d*)",
                r"\g<1>\g<2>#<!-- nolink -->\g<3>",
                body,
                flags=re.MULTILINE,
            )
            .split("\n---")[0]
            .split("<!-- Thank you for")[0]
        )

    def prepare_body_text(self, body: str) -> str:
        return SyncRun.clean_up_body_text(body) + f"\nReviewed in {self.servo_pr}"


@dataclasses.dataclass()
class WPTSync:
    servo_repo: str
    wpt_repo: str
    downstream_wpt_repo: str
    servo_path: str
    wpt_path: str
    github_api_token: str
    github_api_url: str
    github_username: str
    github_email: str
    github_name: str
    suppress_force_push: bool = False

    def __post_init__(self):
        self.servo = GithubRepository(self, self.servo_repo, "main")
        self.wpt = GithubRepository(self, self.wpt_repo, "master")
        self.downstream_wpt = GithubRepository(self, self.downstream_wpt_repo, "master")
        self.local_servo_repo = LocalGitRepo(self.servo_path, self)
        self.local_wpt_repo = LocalGitRepo(self.wpt_path, self)

    def run(self, payload: dict, step_callback=None) -> bool:
        if "pull_request" not in payload:
            return True

        pull_data = payload["pull_request"]
        if NO_SYNC_SIGNAL in pull_data.get("body", ""):
            return True

        # Only look for an existing remote PR if the action is appropriate.
        logging.info("Processing '%s' action...", payload["action"])
        action = payload["action"]
        if action not in ["opened", "synchronize", "reopened", "edited", "closed"]:
            return True

        if (
            action == "edited"
            and "title" not in payload["changes"]
            and "body" not in payload["changes"]
        ):
            return True

        try:
            servo_pr = self.servo.get_pull_request(pull_data["number"])
            downstream_wpt_branch = self.downstream_wpt.get_branch(
                wpt_branch_name_from_servo_pr_number(servo_pr.number)
            )
            upstream_pr = self.wpt.get_open_pull_request_for_branch(
                downstream_wpt_branch
            )
            if upstream_pr:
                logging.info(
                    "  → Detected existing upstream PR %s", upstream_pr)

            run = SyncRun(self, servo_pr, AsyncValue(
                upstream_pr), step_callback)

            pull_data = payload["pull_request"]
            if payload["action"] in ["opened", "synchronize", "reopened"]:
                self.handle_new_pull_request_contents(run, pull_data)
            elif payload["action"] == "edited":
                self.handle_edited_pull_request(run, pull_data)
            elif payload["action"] == "closed":
                self.handle_closed_pull_request(run, pull_data)

            run.run()
            return True
        except Exception as exception:
            if isinstance(exception, subprocess.CalledProcessError):
                logging.error(exception.output)
            logging.error(json.dumps(payload))
            logging.error(exception, exc_info=True)
            return False

    def handle_new_pull_request_contents(self, run: SyncRun, pull_data: dict):
        num_commits = pull_data["commits"]
        head_sha = pull_data["head"]["sha"]
        is_upstreamable = (
            len(
                self.local_servo_repo.run(
                    "diff", head_sha, f"{head_sha}~{num_commits}", "--", UPSTREAMABLE_PATH
                )
            )
            > 0
        )
        logging.info("  → PR is upstreamable: '%s'", is_upstreamable)

        title = pull_data['title']
        body = pull_data['body']
        if run.upstream_pr.has_value():
            if is_upstreamable:
                # In case this is adding new upstreamable changes to a PR that was closed
                # due to a lack of upstreamable changes, force it to be reopened.
                # Github refuses to reopen a PR that had a branch force pushed, so be sure
                # to do this first.
                run.add_step(ChangePRStep(
                    run.upstream_pr.value(), "opened", title, body))
                # Push the relevant changes to the upstream branch.
                run.add_step(CreateOrUpdateBranchForPRStep(
                    pull_data, run.servo_pr))
                run.add_step(CommentStep(
                    run.servo_pr, UPDATED_EXISTING_UPSTREAM_PR))
            else:
                # Close the upstream PR, since would contain no changes otherwise.
                run.add_step(CommentStep(run.upstream_pr.value(),
                             NO_UPSTREAMBLE_CHANGES_COMMENT))
                run.add_step(ChangePRStep(run.upstream_pr.value(), "closed"))
                run.add_step(RemoveBranchForPRStep(pull_data))
                run.add_step(CommentStep(
                    run.servo_pr, CLOSING_EXISTING_UPSTREAM_PR))

        elif is_upstreamable:
            # Push the relevant changes to a new upstream branch.
            branch = run.add_step(
                CreateOrUpdateBranchForPRStep(pull_data, run.servo_pr))

            # Create a pull request against the upstream repository for the new branch.
            assert branch
            upstream_pr = run.add_step(OpenPRStep(
                branch, self.wpt, title, body,
                ["servo-export", "do not merge yet"],
            ))

            assert upstream_pr
            run.upstream_pr = upstream_pr

            # Leave a comment to the new pull request in the original pull request.
            run.add_step(CommentStep(run.servo_pr, OPENED_NEW_UPSTREAM_PR))

    def handle_edited_pull_request(self, run: SyncRun, pull_data: dict):
        logging.info("Changing upstream PR title")
        if run.upstream_pr.has_value():
            run.add_step(ChangePRStep(
                run.upstream_pr.value(
                ), "open", pull_data["title"], pull_data["body"]
            ))
            run.add_step(CommentStep(
                run.servo_pr, UPDATED_TITLE_IN_EXISTING_UPSTREAM_PR))

    def handle_closed_pull_request(self, run: SyncRun, pull_data: dict):
        logging.info("Processing closed PR")
        if not run.upstream_pr.has_value():
            # If we don't recognize this PR, it never contained upstreamable changes.
            return
        if pull_data["merged"]:
            # Since the upstreamable changes have now been merged locally, merge the
            # corresponding upstream PR.
            run.add_step(MergePRStep(
                run.upstream_pr.value(), ["do not merge yet"]))
        else:
            # If a PR with upstreamable changes is closed without being merged, we
            # don't want to merge the changes upstream either.
            run.add_step(ChangePRStep(run.upstream_pr.value(), "closed"))

        # Always clean up our remote branch.
        run.add_step(RemoveBranchForPRStep(pull_data))
