# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# pylint: disable=missing-docstring

import dataclasses
import json
import multiprocessing
import os
import re
import sys
import tempfile
import urllib.error
import urllib.parse
import urllib.request

from typing import List, NamedTuple, Optional, Union, cast, Callable

import mozlog
import mozlog.formatters

from . import SERVO_ROOT, WPT_PATH, WPT_TOOLS_PATH
from .grouping_formatter import ServoFormatter, ServoHandler, UnexpectedResult, UnexpectedSubtestResult
from wptrunner import wptcommandline
from wptrunner import wptrunner


CERTS_PATH = os.path.join(WPT_TOOLS_PATH, "certs")
TRACKER_API = "https://intermittent-tracker.servo.org"
TRACKER_API_ENV_VAR = "INTERMITTENT_TRACKER_API"
TRACKER_DASHBOARD_SECRET_ENV_VAR = "INTERMITTENT_TRACKER_DASHBOARD_SECRET_PROD"
TRACKER_DASHBOARD_MAXIMUM_OUTPUT_LENGTH = 1024


def set_if_none(args: dict, key: str, value: bool | int | str) -> None:
    if key not in args or args[key] is None:
        args[key] = value


def run_tests(default_binary_path: str, **kwargs) -> int:
    print(f"Running WPT tests with {default_binary_path}")

    # By default, Rayon selects the number of worker threads based on the
    # available CPU count. This doesn't work very well when running tests on CI,
    # since we run so many Servo processes in parallel. The result is a lot of
    # extra timeouts. Instead, force Rayon to assume we are running on a 2 CPU
    # environment.
    os.environ["RAYON_RS_NUM_CPUS"] = "2"
    os.environ["RUST_BACKTRACE"] = "1"
    os.environ["HOST_FILE"] = os.path.join(SERVO_ROOT, "tests", "wpt", "hosts")

    # The pytest framework used in the webdriver conformance tests dumps the
    # environment variables when unexpected results occur, and this variable
    # makes CI logs unreadable.
    github_context = os.environ.pop("GITHUB_CONTEXT", None)

    set_if_none(kwargs, "product", "servo")
    set_if_none(kwargs, "config", os.path.join(WPT_PATH, "config.ini"))
    set_if_none(kwargs, "include_manifest", os.path.join(WPT_PATH, "include.ini"))
    set_if_none(kwargs, "manifest_update", False)
    set_if_none(kwargs, "processes", multiprocessing.cpu_count())

    set_if_none(kwargs, "ca_cert_path", os.path.join(CERTS_PATH, "cacert.pem"))
    set_if_none(kwargs, "host_key_path", os.path.join(CERTS_PATH, "web-platform.test.key"))
    set_if_none(kwargs, "host_cert_path", os.path.join(CERTS_PATH, "web-platform.test.pem"))
    # Set `id_hash` as the default chunk, as this better distributes testing across different
    # chunks and leads to more consistent timing on GitHub Actions.
    set_if_none(kwargs, "chunk_type", "id_hash")

    kwargs["user_stylesheets"].append(os.path.join(SERVO_ROOT, "tests", "wpt", "tests", "fonts", "ahem.css"))

    set_if_none(kwargs, "binary", default_binary_path)
    set_if_none(kwargs, "webdriver_binary", default_binary_path)

    if kwargs.pop("rr_chaos"):
        kwargs["debugger"] = "rr"
        kwargs["debugger_args"] = "record --chaos"
        kwargs["repeat_until_unexpected"] = True
        # TODO: Delete rr traces from green test runs?

    prefs = kwargs.pop("prefs")
    kwargs.setdefault("binary_args", [])
    if prefs:
        kwargs["binary_args"] += ["--pref=" + pref for pref in prefs]

    if not kwargs.get("no_default_test_types"):
        test_types = {
            "servo": ["testharness", "reftest", "wdspec", "crashtest"],
            "servodriver": ["testharness", "reftest", "wdspec", "crashtest"],
        }
        product = kwargs.get("product") or "servo"
        kwargs["test_types"] = test_types[product]

    filter_intermittents_output = kwargs.pop("filter_intermittents", None)
    unexpected_raw_log_output_file = kwargs.pop("log_raw_unexpected", None)
    raw_log_outputs = kwargs.get("log_raw", [])
    if filter_intermittents_output and kwargs["retry_unexpected"] <= 0:
        kwargs["retry_unexpected"] = 1

    wptcommandline.check_args(kwargs)

    mozlog.commandline.log_formatters["servo"] = (
        cast(Callable, ServoFormatter),
        "Servo's grouping output formatter",
    )

    use_mach_logging = False
    if len(kwargs["test_list"]) == 1:
        file_ext = os.path.splitext(kwargs["test_list"][0])[1].lower()
        if file_ext in [".htm", ".html", ".js", ".xhtml", ".xht", ".py"]:
            use_mach_logging = True

    if use_mach_logging:
        logger = wptrunner.setup_logging(kwargs, {"mach": sys.stdout})
    else:
        logger = wptrunner.setup_logging(kwargs, {"servo": sys.stdout})

    handler = ServoHandler(detect_flakes=kwargs["retry_unexpected"] >= 1)
    logger.add_handler(handler)

    with tempfile.TemporaryDirectory(prefix="servo-") as config_dir:
        kwargs["binary_args"] += ["--config-dir", config_dir]

        wptrunner.run_tests(**kwargs)

    return_value = int(handler.any_stable_unexpected())

    # Filter intermittents if that was specified on the command-line.
    if filter_intermittents_output:
        if github_context:
            os.environ["GITHUB_CONTEXT"] = github_context

        all_filtered = filter_intermittents(handler.unexpected_results, filter_intermittents_output)
        return_value = 0 if all_filtered else 1

    # Write the unexpected-only raw log if that was specified on the command-line.
    if unexpected_raw_log_output_file:
        if not raw_log_outputs:
            print("'--log-raw-unexpected' not written without '--log-raw'.")
        else:
            write_unexpected_only_raw_log(
                handler.unexpected_results, raw_log_outputs[0].name, unexpected_raw_log_output_file
            )

    return return_value


class GithubContextInformation(NamedTuple):
    build_url: Optional[str]
    pull_url: Optional[str]
    branch_name: Optional[str]


class TrackerDashboardFilter:
    def __init__(self) -> None:
        base_url = os.environ.get(TRACKER_API_ENV_VAR, TRACKER_API)
        self.headers = {"Content-Type": "application/json"}
        if TRACKER_DASHBOARD_SECRET_ENV_VAR in os.environ and os.environ[TRACKER_DASHBOARD_SECRET_ENV_VAR]:
            self.url = f"{base_url}/dashboard/attempts"
            secret = os.environ[TRACKER_DASHBOARD_SECRET_ENV_VAR]
            self.headers["Authorization"] = f"Bearer {secret}"
        else:
            self.url = f"{base_url}/dashboard/query"

    @staticmethod
    def get_github_context_information() -> GithubContextInformation:
        github_context = json.loads(os.environ.get("GITHUB_CONTEXT", "{}"))
        if not github_context:
            return GithubContextInformation(None, None, None)

        repository = github_context["repository"]
        repo_url = f"https://github.com/{repository}"

        run_id = github_context["run_id"]
        build_url = f"{repo_url}/actions/runs/{run_id}"

        commit_title = "<no title>"
        if "merge_group" in github_context["event"]:
            commit_title = github_context["event"]["merge_group"]["head_commit"]["message"]
        if "head_commit" in github_context["event"]:
            commit_title = github_context["event"]["head_commit"]["message"]

        pr_url = None
        match = re.match(r"^Auto merge of #(\d+)", commit_title) or re.match(r"\(#(\d+)\)", commit_title)
        if match:
            pr_url = f"{repo_url}/pull/{match.group(1)}" if match else None

        return GithubContextInformation(build_url, pr_url, github_context["ref_name"])

    def make_data_from_result(
        self,
        result: Union[UnexpectedResult, UnexpectedSubtestResult],
    ) -> dict:
        data = {
            "path": result.path,
            "subtest": None,
            "expected": result.expected,
            "actual": result.actual,
            "time": result.time // 1000,
            # Truncate the message, to avoid issues with lots of output causing "HTTP
            # Error 413: Request Entity Too Large."
            # See https://github.com/servo/servo/issues/31845.
            "message": result.message[0:TRACKER_DASHBOARD_MAXIMUM_OUTPUT_LENGTH],
            "stack": result.stack,
        }
        if isinstance(result, UnexpectedSubtestResult):
            data["subtest"] = result.subtest
        return data

    def report_failures(self, unexpected_results: List[UnexpectedResult]) -> None:
        attempts = []
        for result in unexpected_results:
            attempts.append(self.make_data_from_result(result))
            for subtest_result in result.unexpected_subtest_results:
                attempts.append(self.make_data_from_result(subtest_result))

        context = self.get_github_context_information()
        try:
            request = urllib.request.Request(
                url=self.url,
                method="POST",
                data=json.dumps(
                    {
                        "branch": context.branch_name,
                        "build_url": context.build_url,
                        "pull_url": context.pull_url,
                        "attempts": attempts,
                    }
                ).encode("utf-8"),
                headers=self.headers,
            )

            known_intermittents = dict()
            with urllib.request.urlopen(request) as response:
                for test in json.load(response)["known"]:
                    known_intermittents[test["path"]] = [issue["number"] for issue in test["issues"]]

        except urllib.error.HTTPError as e:
            print(e)
            print(e.readlines())
            raise (e)

        for result in unexpected_results:
            result.issues = known_intermittents.get(result.path, [])


def filter_intermittents(unexpected_results: List[UnexpectedResult], output_path: str) -> bool:
    dashboard = TrackerDashboardFilter()
    print(f"Filtering {len(unexpected_results)} unexpected results for known intermittents via <{dashboard.url}>")
    dashboard.report_failures(unexpected_results)

    def add_result(output: list[str], text: str, results: List[UnexpectedResult], filter_func) -> None:
        filtered = [str(result) for result in filter(filter_func, results)]
        if filtered:
            output += [f"{text} ({len(filtered)}): ", *filtered]

    def is_stable_and_unexpected(result: UnexpectedResult) -> bool:
        return not result.flaky and not result.issues

    output: List[str] = []
    add_result(output, "Flaky unexpected results", unexpected_results, lambda result: result.flaky)
    add_result(
        output,
        "Stable unexpected results that are known-intermittent",
        unexpected_results,
        lambda result: not result.flaky and result.issues,
    )
    add_result(output, "Stable unexpected results", unexpected_results, is_stable_and_unexpected)
    print("\n".join(output))

    with open(output_path, "w", encoding="utf-8") as file:
        json.dump([dataclasses.asdict(result) for result in unexpected_results], file)

    return not any([is_stable_and_unexpected(result) for result in unexpected_results])


def write_unexpected_only_raw_log(
    unexpected_results: List[UnexpectedResult], raw_log_file: str, filtered_raw_log_file: str
) -> None:
    tests = [result.path for result in unexpected_results]
    print(f"Writing unexpected-only raw log to {filtered_raw_log_file}")

    with open(filtered_raw_log_file, "w", encoding="utf-8") as output:
        with open(raw_log_file) as input:
            for line in input.readlines():
                data = json.loads(line)
                if data["action"] in ["suite_start", "suite_end", "add_subsuite"] or (
                    "test" in data and data["test"] in tests
                ):
                    output.write(line)
