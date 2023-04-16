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

import logging
import os
import textwrap

from typing import TYPE_CHECKING, Generic, Optional, TypeVar

from .common import COULD_NOT_APPLY_CHANGES_DOWNSTREAM_COMMENT
from .common import COULD_NOT_APPLY_CHANGES_UPSTREAM_COMMENT
from .common import COULD_NOT_MERGE_CHANGES_DOWNSTREAM_COMMENT
from .common import COULD_NOT_MERGE_CHANGES_UPSTREAM_COMMENT
from .common import UPSTREAMABLE_PATH
from .common import wpt_branch_name_from_servo_pr_number
from .github import GithubBranch, GithubRepository, PullRequest

if TYPE_CHECKING:
    from . import SyncRun, WPTSync

PATCH_FILE_NAME = "tmp.patch"


class Step:
    def __init__(self, name):
        self.name = name

    def provides(self) -> Optional[AsyncValue]:
        return None

    def run(self, _: SyncRun):
        return


T = TypeVar('T')


class AsyncValue(Generic[T]):
    def __init__(self, value: Optional[T] = None):
        self._value = value

    def resolve(self, value: T):
        self._value = value

    def value(self) -> T:
        assert self._value is not None
        return self._value

    def has_value(self):
        return self._value is not None


class CreateOrUpdateBranchForPRStep(Step):
    def __init__(self, pull_data: dict, pull_request: PullRequest):
        Step.__init__(self, "CreateOrUpdateBranchForPRStep")
        self.pull_data = pull_data
        self.pull_request = pull_request
        self.branch: AsyncValue[GithubBranch] = AsyncValue()

    def provides(self):
        return self.branch

    def run(self, run: SyncRun):
        try:
            commits = self._get_upstreamable_commits_from_local_servo_repo(
                run.sync)
            branch_name = self._create_or_update_branch_for_pr(run, commits)
            branch = run.sync.downstream_wpt.get_branch(branch_name)

            self.branch.resolve(branch)
            self.name += f":{len(commits)}:{branch}"
        except Exception as exception:
            logging.info("Could not apply changes to upstream WPT repository.")
            logging.info(exception, exc_info=True)

            run.steps = []
            run.add_step(CommentStep(
                self.pull_request, COULD_NOT_APPLY_CHANGES_DOWNSTREAM_COMMENT
            ))
            if run.upstream_pr.has_value():
                run.add_step(CommentStep(
                    run.upstream_pr.value(), COULD_NOT_APPLY_CHANGES_UPSTREAM_COMMENT
                ))

    def _get_upstreamable_commits_from_local_servo_repo(self, sync: WPTSync):
        local_servo_repo = sync.local_servo_repo
        number_of_commits = self.pull_data["commits"]
        pr_head = self.pull_data["head"]["sha"]
        commit_shas = local_servo_repo.run(
            "log", "--pretty=%H", pr_head, f"-{number_of_commits}"
        ).splitlines()

        filtered_commits = []
        for sha in commit_shas:
            # Specifying the path here does a few things. First, it excludes any
            # changes that do not touch WPT files at all. Secondly, when a file is
            # moved in or out of the WPT directory the filename which is outside the
            # directory becomes /dev/null and the change becomes an addition or
            # deletion. This makes the patch usable on the WPT repository itself.
            # TODO: If we could cleverly parse and manipulate the full commit diff
            # we could avoid cloning the servo repository altogether and only
            # have to fetch the commit diffs from GitHub.
            # NB: The output of git show might include binary files or non-UTF8 text,
            # so store the content of the diff as a `bytes`.
            diff = local_servo_repo.run_without_encoding(
                "show", "--binary", "--format=%b", sha, "--", UPSTREAMABLE_PATH
            )

            # Retrieve the diff of any changes to files that are relevant
            if diff:
                # Create an object that contains everything necessary to transplant this
                # commit to another repository.
                filtered_commits += [
                    {
                        "author": local_servo_repo.run(
                            "show", "-s", "--pretty=%an <%ae>", sha
                        ),
                        "message": local_servo_repo.run(
                            "show", "-s", "--pretty=%B", sha
                        ),
                        "diff": diff,
                    }
                ]
        return filtered_commits

    def _apply_filtered_servo_commit_to_wpt(self, run: SyncRun, commit: dict):
        patch_path = os.path.join(run.sync.wpt_path, PATCH_FILE_NAME)
        strip_count = UPSTREAMABLE_PATH.count("/") + 1

        try:
            with open(patch_path, "wb") as file:
                file.write(commit["diff"])
            run.sync.local_wpt_repo.run(
                "apply", PATCH_FILE_NAME, "-p", str(strip_count)
            )
        finally:
            # Ensure the patch file is not added with the other changes.
            os.remove(patch_path)

        run.sync.local_wpt_repo.run("add", "--all")
        run.sync.local_wpt_repo.run(
            "commit", "--message", commit["message"], "--author", commit["author"]
        )

    def _create_or_update_branch_for_pr(
        self, run: SyncRun, commits: list[dict], pre_commit_callback=None
    ):
        branch_name = wpt_branch_name_from_servo_pr_number(
            self.pull_data["number"])
        try:
            # Create a new branch with a unique name that is consistent between
            # updates of the same PR.
            run.sync.local_wpt_repo.run("checkout", "-b", branch_name)

            for commit in commits:
                self._apply_filtered_servo_commit_to_wpt(run, commit)

            if pre_commit_callback:
                pre_commit_callback()

            # Push the branch upstream (forcing to overwrite any existing changes).
            if not run.sync.suppress_force_push:

                # In order to push to our downstream branch we need to ensure that
                # the local repository isn't a shallow clone. Shallow clones are
                # commonly created by GitHub actions.
                run.sync.local_wpt_repo.run("fetch", "--unshallow", "origin")

                user = run.sync.github_username
                token = run.sync.github_api_token
                repo = run.sync.downstream_wpt_repo
                remote_url = f"https://{user}:{token}@github.com/{repo}.git"
                run.sync.local_wpt_repo.run(
                    "push", "-f", remote_url, branch_name)

            return branch_name
        finally:
            try:
                run.sync.local_wpt_repo.run("checkout", "master")
                run.sync.local_wpt_repo.run("branch", "-D", branch_name)
            except Exception:
                pass


class RemoveBranchForPRStep(Step):
    def __init__(self, pull_request):
        Step.__init__(self, "RemoveBranchForPRStep")
        self.branch_name = wpt_branch_name_from_servo_pr_number(
            pull_request["number"])

    def run(self, run: SyncRun):
        self.name += f":{run.sync.downstream_wpt.get_branch(self.branch_name)}"
        logging.info("  -> Removing branch used for upstream PR")
        if not run.sync.suppress_force_push:
            user = run.sync.github_username
            token = run.sync.github_api_token
            repo = run.sync.downstream_wpt_repo
            remote_url = f"https://{user}:{token}@github.com/{repo}.git"
            run.sync.local_wpt_repo.run("push", remote_url, "--delete",
                                        self.branch_name)


class ChangePRStep(Step):
    def __init__(
        self,
        pull_request: PullRequest,
        state: str,
        title: Optional[str] = None,
        body: Optional[str] = None,
    ):
        name = f"ChangePRStep:{pull_request}:{state}"
        if title:
            name += f":{title}"

        Step.__init__(self, name)
        self.pull_request = pull_request
        self.state = state
        self.title = title
        self.body = body

    def run(self, run: SyncRun):
        body = self.body
        if body:
            body = run.prepare_body_text(body)
            self.name += (
                f':{textwrap.shorten(body, width=20, placeholder="...")}[{len(body)}]'
            )

        self.pull_request.change(state=self.state, title=self.title, body=body)


class MergePRStep(Step):
    def __init__(self, pull_request: PullRequest, labels_to_remove: list[str] = []):
        Step.__init__(self, f"MergePRStep:{pull_request}")
        self.pull_request = pull_request
        self.labels_to_remove = labels_to_remove

    def run(self, run: SyncRun):
        for label in self.labels_to_remove:
            self.pull_request.remove_label(label)
        try:
            self.pull_request.merge()
        except Exception as exception:
            logging.warning("Could not merge PR (%s).", self.pull_request)
            logging.warning(exception, exc_info=True)

            run.steps = []
            run.add_step(CommentStep(
                self.pull_request, COULD_NOT_MERGE_CHANGES_UPSTREAM_COMMENT
            ))
            run.add_step(CommentStep(
                run.servo_pr, COULD_NOT_MERGE_CHANGES_DOWNSTREAM_COMMENT
            ))
            self.pull_request.add_labels(["stale-servo-export"])


class OpenPRStep(Step):
    def __init__(
        self,
        source_branch: AsyncValue[GithubBranch],
        target_repo: GithubRepository,
        title: str,
        body: str,
        labels: list[str],
    ):
        Step.__init__(self, "OpenPRStep")
        self.title = title
        self.body = body
        self.source_branch = source_branch
        self.target_repo = target_repo
        self.new_pr: AsyncValue[PullRequest] = AsyncValue()
        self.labels = labels

    def provides(self):
        return self.new_pr

    def run(self, run: SyncRun):
        pull_request = self.target_repo.open_pull_request(
            self.source_branch.value(), self.title, run.prepare_body_text(self.body)
        )

        if self.labels:
            pull_request.add_labels(self.labels)

        self.new_pr.resolve(pull_request)

        self.name += f":{self.source_branch.value()}→{self.new_pr.value()}"


class CommentStep(Step):
    def __init__(self, pull_request: PullRequest, comment_template: str):
        Step.__init__(self, "CommentStep")
        self.pull_request = pull_request
        self.comment_template = comment_template

    def run(self, run: SyncRun):
        comment = run.make_comment(self.comment_template)
        self.name += f":{self.pull_request}:{comment}"
        self.pull_request.leave_comment(comment)
