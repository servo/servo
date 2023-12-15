#!/usr/bin/env python

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
# pylint: disable=global-statement
# pylint: disable=line-too-long
# pylint: disable=missing-docstring
# pylint: disable=protected-access

# This allows using types that are defined later in the file.
from __future__ import annotations

import dataclasses
import json
import locale
import logging
import os
import shutil
import subprocess
import tempfile
import threading
import time
import unittest

from functools import partial
from typing import Any, Optional, Tuple, Type
from wsgiref.simple_server import WSGIRequestHandler, make_server

import flask
import flask.cli
import requests

from .exporter import SyncRun, WPTSync
from .exporter.step import CreateOrUpdateBranchForPRStep

TESTS_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "tests")
SYNC: Optional[WPTSync] = None
TMP_DIR: Optional[str] = None
PORT = 9000


@dataclasses.dataclass
class MockPullRequest():
    head: str
    number: int
    state: str = "open"


class MockGitHubAPIServer():
    def __init__(self, port: int):
        self.port = port
        self.disable_logging()
        self.app = flask.Flask(__name__)
        self.pulls: list[MockPullRequest] = []

        class NoLoggingHandler(WSGIRequestHandler):
            def log_message(self, *args):
                pass
        if logging.getLogger().level == logging.DEBUG:
            handler = WSGIRequestHandler
        else:
            handler = NoLoggingHandler

        self.server = make_server('localhost', self.port, self.app, handler_class=handler)
        self.start_server_thread()

    def disable_logging(self):
        flask.cli.show_server_banner = lambda *args: None
        logging.getLogger("werkzeug").disabled = True
        logging.getLogger('werkzeug').setLevel(logging.CRITICAL)

    def start(self):
        self.thread.start()

        # Wait for the server to be started.
        while True:
            try:
                response = requests.get(f'http://localhost:{self.port}/ping', timeout=1)
                assert response.status_code == 200
                assert response.text == 'pong'
                break
            except Exception:
                time.sleep(0.1)

    def reset_server_state_with_pull_requests(self, pulls: list[MockPullRequest]):
        response = requests.get(
            f'http://localhost:{self.port}/reset-mock-github',
            json=[dataclasses.asdict(pull_request) for pull_request in pulls],
            timeout=1
        )
        assert response.status_code == 200
        assert response.text == '👍'

    def shutdown(self):
        self.server.shutdown()
        self.thread.join()

    def start_server_thread(self):
        # pylint: disable=unused-argument
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

        @self.app.route("/ping")
        def ping():
            return ('pong', 200)

        @self.app.route("/reset-mock-github")
        def reset_server():
            self.pulls = [
                MockPullRequest(pull_request['head'],
                                pull_request['number'],
                                pull_request['state'])
                for pull_request in flask.request.json]
            return ('👍', 200)

        @self.app.route("/repos/<org>/<repo>/pulls/<int:number>/merge", methods=['PUT'])
        def merge_pull_request(org, repo, number):
            for pull_request in self.pulls:
                if pull_request.number == number:
                    pull_request.state = 'closed'
                    return ('', 204)
            return ('', 404)

        @self.app.route("/search/issues", methods=['GET'])
        def search():
            params = {}
            param_strings = flask.request.args.get("q", "").split(" ")
            for string in param_strings:
                parts = string.split(":")
                params[parts[0]] = parts[1]

            assert params["is"] == "pr"
            assert params["state"] == "open"
            assert "author" in params
            assert "head" in params
            head_ref = f"{params['author']}:{params['head']}"

            for pull_request in self.pulls:
                if pull_request.head == head_ref:
                    return json.dumps({
                        "total_count": 1,
                        "items": [{
                            "number": pull_request.number
                        }]
                    })
            return json.dumps({"total_count": 0, "items": []})

        @self.app.route("/repos/<org>/<repo>/pulls", methods=['POST'])
        def create_pull_request(org, repo):
            new_pr_number = len(self.pulls) + 1
            self.pulls.append(MockPullRequest(
                flask.request.json["head"],
                new_pr_number,
                "open"
            ))
            return {"number": new_pr_number}

        @self.app.route("/repos/<org>/<repo>/pulls/<int:number>", methods=['PATCH'])
        def update_pull_request(org, repo, number):
            for pull_request in self.pulls:
                if pull_request.number == number:
                    if 'state' in flask.request.json:
                        pull_request.state = flask.request.json['state']
                    return ('', 204)
            return ('', 404)

        @self.app.route("/repos/<org>/<repo>/issues/<number>/labels", methods=['GET', 'POST'])
        @self.app.route("/repos/<org>/<repo>/issues/<number>/labels/<label>", methods=['DELETE'])
        @self.app.route("/repos/<org>/<repo>/issues/<issue>/comments", methods=['GET', 'POST'])
        def other_requests(*args, **kwargs):
            return ('', 204)


class TestCleanUpBodyText(unittest.TestCase):
    """Tests that SyncRun.clean_up_body_text properly prepares the
    body text for an upstream pull request."""

    def test_prepare_body(self):
        text = "Simple body text"
        self.assertEqual(text, SyncRun.clean_up_body_text(text))
        self.assertEqual(
            "With reference: #<!-- nolink -->3",
            SyncRun.clean_up_body_text("With reference: #3"),
        )
        self.assertEqual(
            "Multi reference: #<!-- nolink -->3 and #<!-- nolink -->1020",
            SyncRun.clean_up_body_text("Multi reference: #3 and #1020"),
        )
        self.assertEqual(
            "Subject\n\nBody text #<!-- nolink -->1",
            SyncRun.clean_up_body_text(
                "Subject\n\nBody text #1\n---<!-- Thank you for contributing"
            ),
        )
        self.assertEqual(
            "Subject\n\nNo dashes",
            SyncRun.clean_up_body_text(
                "Subject\n\nNo dashes<!-- Thank you for contributing"
            ),
        )
        self.assertEqual(
            "Subject\n\nNo --- comment",
            SyncRun.clean_up_body_text(
                "Subject\n\nNo --- comment\n---Other stuff that"
            ),
        )
        self.assertEqual(
            "Subject\n\n#<!-- nolink -->3 servo#<!-- nolink -->3 servo/servo#3",
            SyncRun.clean_up_body_text(
                "Subject\n\n#3 servo#3 servo/servo#3",
            ),
            "Only relative and bare issue reference links should be escaped."
        )


class TestApplyCommitsToWPT(unittest.TestCase):
    """Tests that commits are properly applied to WPT by
    CreateOrUpdateBranchForPRStep._create_or_update_branch_for_pr."""

    def run_test(self, pr_number: int, commit_data: dict):
        def make_commit(data):
            with open(os.path.join(TESTS_DIR, data[2]), "rb") as file:
                return {"author": data[0], "message": data[1], "diff": file.read()}

        commits = [make_commit(data) for data in commit_data]

        assert SYNC is not None
        pull_request = SYNC.servo.get_pull_request(pr_number)
        step = CreateOrUpdateBranchForPRStep({"number": pr_number}, pull_request)

        def get_applied_commits(
            num_commits: int, applied_commits: list[Tuple[str, str]]
        ):
            assert SYNC is not None
            repo = SYNC.local_wpt_repo
            log = ["log", "--oneline", f"-{num_commits}"]
            applied_commits += list(
                zip(
                    repo.run(*log, "--format=%aN <%ae>").splitlines(),
                    repo.run(*log, "--format=%s").splitlines(),
                )
            )
            applied_commits.reverse()

        applied_commits: list[Any] = []
        callback = partial(get_applied_commits, len(commits), applied_commits)
        step._create_or_update_branch_for_pr(
            SyncRun(SYNC, pull_request, None, None), commits, callback
        )

        expected_commits = [(commit["author"], commit["message"]) for commit in commits]
        self.assertListEqual(applied_commits, expected_commits)

    def test_simple_commit(self):
        self.run_test(
            45, [["test author <test@author>", "test commit message", "18746.diff"]]
        )

    def test_two_commits(self):
        self.run_test(
            100,
            [
                ["test author <test@author>", "test commit message", "18746.diff"],
                ["another person <two@author>", "a different message", "wpt.diff"],
                ["another person <two@author>", "adding some non-utf8 chaos", "add-non-utf8-file.diff"],
            ],
        )

    def test_non_utf8_commit(self):
        self.run_test(
            100,
            [
                ["test author <nonutf8@author>", "adding some non-utf8 chaos", "add-non-utf8-file.diff"],
            ],
        )


class TestFullSyncRun(unittest.TestCase):
    server: Optional[MockGitHubAPIServer] = None

    @classmethod
    def setUpClass(cls):
        cls.server = MockGitHubAPIServer(PORT)

    @classmethod
    def tearDownClass(cls):
        assert cls.server is not None
        cls.server.shutdown()

    def tearDown(self):
        assert SYNC is not None

        # Clean up any old files.
        first_commit_hash = SYNC.local_servo_repo.run("rev-list", "HEAD").splitlines()[
            -1
        ]
        SYNC.local_servo_repo.run("reset", "--hard", first_commit_hash)
        SYNC.local_servo_repo.run("clean", "-fxd")

    def mock_servo_repository_state(self, diffs: list):
        assert SYNC is not None

        def make_commit_data(diff):
            if not isinstance(diff, list):
                return [diff, "tmp author", "tmp@tmp.com", "tmp commit message"]
            return diff

        # Apply each commit to the repository.
        orig_sha = SYNC.local_servo_repo.run("rev-parse", "HEAD").strip()
        commits = [make_commit_data(diff) for diff in diffs]
        for commit in commits:
            patch_file, author, email, message = commit
            SYNC.local_servo_repo.run("apply", os.path.join(TESTS_DIR, patch_file))
            SYNC.local_servo_repo.run("add", ".")
            SYNC.local_servo_repo.run(
                "commit",
                "-a",
                "--author",
                f"{author} <{email}>",
                "-m",
                message,
                env={
                    "GIT_COMMITTER_NAME": author.encode(locale.getpreferredencoding()),
                    "GIT_COMMITTER_EMAIL": email,
                },
            )

        # Reset the repository to the original hash, but the commits are still
        # available until the next `git gc`.
        last_commit_sha = SYNC.local_servo_repo.run("rev-parse", "HEAD").strip()
        SYNC.local_servo_repo.run("reset", "--hard", orig_sha)
        return last_commit_sha

    def run_test(
        self, payload_file: str, diffs: list, existing_prs: list[MockPullRequest] = []
    ):
        with open(os.path.join(TESTS_DIR, payload_file), encoding="utf-8") as file:
            payload = json.loads(file.read())

        logging.info("Mocking application of PR to servo.")
        last_commit_sha = self.mock_servo_repository_state(diffs)
        payload["pull_request"]["head"]["sha"] = last_commit_sha

        logging.info("Resetting server state")
        assert self.server is not None
        self.server.reset_server_state_with_pull_requests(existing_prs)

        actual_steps = []
        assert SYNC is not None
        SYNC.run(payload, step_callback=lambda step: actual_steps.append(step.name))
        return actual_steps

    def test_opened_upstreamable_pr(self):
        self.assertListEqual(
            self.run_test("opened.json", ["18746.diff"]),
            [
                "CreateOrUpdateBranchForPRStep:1:servo-wpt-sync/wpt/servo_export_18746",
                "OpenPRStep:servo-wpt-sync/wpt/servo_export_18746→wpt/wpt#1",
                "CommentStep:servo/servo#18746:🤖 Opened new upstream WPT pull request "
                "(wpt/wpt#1) with upstreamable changes.",
            ],
        )

    def test_opened_upstreamable_pr_with_move_into_wpt(self):
        self.assertListEqual(
            self.run_test("opened.json", ["move-into-wpt.diff"]),
            [
                "CreateOrUpdateBranchForPRStep:1:servo-wpt-sync/wpt/servo_export_18746",
                "OpenPRStep:servo-wpt-sync/wpt/servo_export_18746→wpt/wpt#1",
                "CommentStep:servo/servo#18746:🤖 Opened new upstream WPT pull request "
                "(wpt/wpt#1) with upstreamable changes.",
            ],
        )

    def test_opened_upstreamble_pr_with_move_into_wpt_and_non_ascii_author(self):
        self.assertListEqual(
            self.run_test(
                "opened.json",
                [
                    [
                        "move-into-wpt.diff",
                        "Fernando Jiménez Moreno",
                        "foo@bar.com",
                        "ééééé",
                    ]
                ],
            ),
            [
                "CreateOrUpdateBranchForPRStep:1:servo-wpt-sync/wpt/servo_export_18746",
                "OpenPRStep:servo-wpt-sync/wpt/servo_export_18746→wpt/wpt#1",
                "CommentStep:servo/servo#18746:🤖 Opened new upstream WPT pull request "
                "(wpt/wpt#1) with upstreamable changes.",
            ],
        )

    def test_opened_upstreamable_pr_with_move_out_of_wpt(self):
        self.assertListEqual(
            self.run_test("opened.json", ["move-out-of-wpt.diff"]),
            [
                "CreateOrUpdateBranchForPRStep:1:servo-wpt-sync/wpt/servo_export_18746",
                "OpenPRStep:servo-wpt-sync/wpt/servo_export_18746→wpt/wpt#1",
                "CommentStep:servo/servo#18746:🤖 Opened new upstream WPT pull request "
                "(wpt/wpt#1) with upstreamable changes.",
            ],
        )

    def test_opened_new_mr_with_no_sync_signal(self):
        self.assertListEqual(
            self.run_test("opened-with-no-sync-signal.json", ["18746.diff"]), []
        )
        self.assertListEqual(
            self.run_test("opened-with-no-sync-signal.json", ["non-wpt.diff"]), []
        )

    def test_opened_upstreamable_pr_not_applying_cleanly_to_upstream(self):
        self.assertListEqual(
            self.run_test("opened.json", ["does-not-apply-cleanly.diff"]),
            [
                "CreateOrUpdateBranchForPRStep",
                "CommentStep:servo/servo#18746:🛠 These changes could not be applied "
                "onto the latest upstream WPT. Servo's copy of the Web Platform Tests may be out of sync.",
            ],
        )

    def test_open_new_upstreamable_pr_with_preexisting_upstream_pr(self):
        self.assertListEqual(
            self.run_test(
                "opened.json",
                ["18746.diff"],
                [MockPullRequest("servo-wpt-sync:servo_export_18746", 1)],
            ),
            [
                "ChangePRStep:wpt/wpt#1:opened:This is a test:<!-- Please...[95]",
                "CreateOrUpdateBranchForPRStep:1:servo-wpt-sync/wpt/servo_export_18746",
                "CommentStep:servo/servo#18746:📝 Transplanted new upstreamable changes to "
                "existing upstream WPT pull request (wpt/wpt#1).",
            ],
        )

    def test_open_new_non_upstreamable_pr_with_preexisting_upstream_pr(self):
        self.assertListEqual(
            self.run_test(
                "opened.json",
                ["non-wpt.diff"],
                [MockPullRequest("servo-wpt-sync:servo_export_18746", 1)],
            ),
            [
                "CommentStep:wpt/wpt#1:👋 Downstream pull request (servo/servo#18746) no longer "
                "contains any upstreamable changes. Closing pull request without merging.",
                "ChangePRStep:wpt/wpt#1:closed",
                "RemoveBranchForPRStep:servo-wpt-sync/wpt/servo_export_18746",
                "CommentStep:servo/servo#18746:🤖 This change no longer contains upstreamable changes "
                "to WPT; closed existing upstream pull request (wpt/wpt#1).",
            ]
        )

    def test_opened_upstreamable_pr_with_non_utf8_file_contents(self):
        self.assertListEqual(
            self.run_test("opened.json", ["add-non-utf8-file.diff"]),
            [
                "CreateOrUpdateBranchForPRStep:1:servo-wpt-sync/wpt/servo_export_18746",
                "OpenPRStep:servo-wpt-sync/wpt/servo_export_18746→wpt/wpt#1",
                "CommentStep:servo/servo#18746:🤖 Opened new upstream WPT pull request "
                "(wpt/wpt#1) with upstreamable changes.",
            ],
        )

    def test_open_new_upstreamable_pr_with_preexisting_upstream_pr_not_apply_cleanly_to_upstream(
        self,
    ):
        self.assertListEqual(
            self.run_test(
                "opened.json",
                ["does-not-apply-cleanly.diff"],
                [MockPullRequest("servo-wpt-sync:servo_export_18746", 1)],
            ),
            [
                "ChangePRStep:wpt/wpt#1:opened:This is a test:<!-- Please...[95]",
                "CreateOrUpdateBranchForPRStep",
                "CommentStep:servo/servo#18746:🛠 These changes could not be applied onto the latest "
                "upstream WPT. Servo's copy of the Web Platform Tests may be out of sync.",
                "CommentStep:wpt/wpt#1:🛠 Changes from the source pull request (servo/servo#18746) can "
                "no longer be cleanly applied. Waiting for a new version of these changes downstream.",
            ],
        )

    def test_closed_pr_no_upstream_pr(self):
        self.assertListEqual(self.run_test("closed.json", ["18746.diff"]), [])

    def test_closed_pr_with_preexisting_upstream_pr(self):
        self.assertListEqual(
            self.run_test(
                "closed.json",
                ["18746.diff"],
                [MockPullRequest("servo-wpt-sync:servo_export_18746", 10)],
            ),
            [
                "ChangePRStep:wpt/wpt#10:closed",
                "RemoveBranchForPRStep:servo-wpt-sync/wpt/servo_export_18746"
            ]
        )

    def test_synchronize_move_new_changes_to_preexisting_upstream_pr(self):
        self.assertListEqual(
            self.run_test(
                "synchronize.json",
                ["18746.diff"],
                [MockPullRequest("servo-wpt-sync:servo_export_19612", 10)],
            ),
            [
                "ChangePRStep:wpt/wpt#10:opened:deny warnings:<!-- Please...[142]",
                "CreateOrUpdateBranchForPRStep:1:servo-wpt-sync/wpt/servo_export_19612",
                "CommentStep:servo/servo#19612:📝 Transplanted new upstreamable changes to existing "
                "upstream WPT pull request (wpt/wpt#10).",
            ]
        )

    def test_synchronize_close_upstream_pr_after_new_changes_do_not_include_wpt(self):
        self.assertListEqual(
            self.run_test(
                "synchronize.json",
                ["non-wpt.diff"],
                [MockPullRequest("servo-wpt-sync:servo_export_19612", 11)],
            ),
            [
                "CommentStep:wpt/wpt#11:👋 Downstream pull request (servo/servo#19612) no longer contains any "
                "upstreamable changes. Closing pull request without merging.",
                "ChangePRStep:wpt/wpt#11:closed",
                "RemoveBranchForPRStep:servo-wpt-sync/wpt/servo_export_19612",
                "CommentStep:servo/servo#19612:🤖 This change no longer contains upstreamable changes to WPT; "
                "closed existing upstream pull request (wpt/wpt#11).",
            ]
        )

    def test_synchronize_open_upstream_pr_after_new_changes_include_wpt(self):
        self.assertListEqual(
            self.run_test("synchronize.json", ["18746.diff"]),
            [
                "CreateOrUpdateBranchForPRStep:1:servo-wpt-sync/wpt/servo_export_19612",
                "OpenPRStep:servo-wpt-sync/wpt/servo_export_19612→wpt/wpt#1",
                "CommentStep:servo/servo#19612:🤖 Opened new upstream WPT pull request "
                "(wpt/wpt#1) with upstreamable changes.",
            ]
        )

    def test_synchronize_fail_to_update_preexisting_pr_after_new_changes_do_not_apply(
        self,
    ):
        self.assertListEqual(
            self.run_test(
                "synchronize.json",
                ["does-not-apply-cleanly.diff"],
                [MockPullRequest("servo-wpt-sync:servo_export_19612", 11)],
            ),
            [
                "ChangePRStep:wpt/wpt#11:opened:deny warnings:<!-- Please...[142]",
                "CreateOrUpdateBranchForPRStep",
                "CommentStep:servo/servo#19612:🛠 These changes could not be applied onto the "
                "latest upstream WPT. Servo's copy of the Web Platform Tests may be out of sync.",
                "CommentStep:wpt/wpt#11:🛠 Changes from the source pull request (servo/servo#19612) can "
                "no longer be cleanly applied. Waiting for a new version of these changes downstream.",
            ]
        )

    def test_edited_with_upstream_pr(self):
        self.assertListEqual(
            self.run_test(
                "edited.json", ["wpt.diff"],
                [MockPullRequest("servo-wpt-sync:servo_export_19620", 10)]
            ),
            [
                "ChangePRStep:wpt/wpt#10:open:A cool new title:Reference #<!--...[136]",
                "CommentStep:servo/servo#19620:✍ Updated existing upstream WPT pull "
                "request (wpt/wpt#10) title and body."
            ]
        )

    def test_edited_with_no_upstream_pr(self):
        self.assertListEqual(self.run_test("edited.json", ["wpt.diff"], []), [])

    def test_synchronize_move_new_changes_to_preexisting_upstream_pr_with_multiple_commits(
        self,
    ):
        self.assertListEqual(
            self.run_test(
                "synchronize-multiple.json", ["18746.diff", "non-wpt.diff", "wpt.diff"]
            ),
            [
                "CreateOrUpdateBranchForPRStep:2:servo-wpt-sync/wpt/servo_export_19612",
                "OpenPRStep:servo-wpt-sync/wpt/servo_export_19612→wpt/wpt#1",
                "CommentStep:servo/servo#19612:"
                "🤖 Opened new upstream WPT pull request (wpt/wpt#1) with upstreamable changes.",
            ]
        )

    def test_synchronize_with_non_upstreamable_changes(self):
        self.assertListEqual(self.run_test("synchronize.json", ["non-wpt.diff"]), [])

    def test_merge_upstream_pr_after_merge(self):
        self.assertListEqual(
            self.run_test(
                "merged.json",
                ["18746.diff"],
                [MockPullRequest("servo-wpt-sync:servo_export_19620", 100)]
            ),
            [
                "MergePRStep:wpt/wpt#100",
                "RemoveBranchForPRStep:servo-wpt-sync/wpt/servo_export_19620"
            ]
        )

    def test_pr_merged_no_upstream_pr(self):
        self.assertListEqual(self.run_test("merged.json", ["18746.diff"]), [])

    def test_merge_of_non_upstreamble_pr(self):
        self.assertListEqual(self.run_test("merged.json", ["non-wpt.diff"]), [])


def setUpModule():
    # pylint: disable=invalid-name
    global TMP_DIR, SYNC

    TMP_DIR = tempfile.mkdtemp()
    SYNC = WPTSync(
        servo_repo="servo/servo",
        wpt_repo="wpt/wpt",
        downstream_wpt_repo="servo-wpt-sync/wpt",
        servo_path=os.path.join(TMP_DIR, "servo-mock"),
        wpt_path=os.path.join(TMP_DIR, "wpt-mock"),
        github_api_token="",
        github_api_url=f"http://localhost:{PORT}",
        github_username="servo-wpt-sync",
        github_email="servo-wpt-sync",
        github_name="servo-wpt-sync@servo.org",
        suppress_force_push=True,
    )

    def setup_mock_repo(repo_name, local_repo, default_branch: str):
        subprocess.check_output(
            ["cp", "-R", "-p", os.path.join(TESTS_DIR, repo_name), local_repo.path])
        local_repo.run("init", "-b", default_branch)
        local_repo.run("add", ".")
        local_repo.run("commit", "-a", "-m", "Initial commit")

    logging.info("=" * 80)
    logging.info("Setting up mock repositories")
    setup_mock_repo("servo-mock", SYNC.local_servo_repo, "main")
    setup_mock_repo("wpt-mock", SYNC.local_wpt_repo, "master")
    logging.info("=" * 80)


def tearDownModule():
    # pylint: disable=invalid-name
    shutil.rmtree(TMP_DIR)


def run_tests():
    verbosity = 1 if logging.getLogger().level >= logging.WARN else 2

    def run_suite(test_case: Type[unittest.TestCase]):
        return unittest.TextTestRunner(verbosity=verbosity).run(
            unittest.TestLoader().loadTestsFromTestCase(test_case)
        ).wasSuccessful()

    return all([
        run_suite(TestApplyCommitsToWPT),
        run_suite(TestCleanUpBodyText),
        run_suite(TestFullSyncRun),
    ])
