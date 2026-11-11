# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# pylint: disable=missing-docstring

import dataclasses
import json
import multiprocessing
import os
import subprocess
import sys
import tempfile

from typing import List, cast, Any
from collections.abc import Callable

import mozlog
import mozlog.formatters

from . import SERVO_ROOT, WPT_PATH, WPT_TOOLS_PATH
from .grouping_formatter import ServoFormatter, ServoHandler, UnexpectedResult
from .manifestupdate import ensure_manifests_up_to_date
from wptrunner import wptcommandline
from wptrunner import wptrunner


CERTS_PATH = os.path.join(WPT_TOOLS_PATH, "certs")
INTERMITTENT_ISSUES_GRAPHQL = """
query($endCursor: String) {
  search(query: "repo:servo/servo is:issue is:open label:I-intermittent -label:C-disabled",
         type: ISSUE, first: 100, after: $endCursor) {
    pageInfo { hasNextPage endCursor }
    nodes { ... on Issue { number title } }
  }
}
"""


def set_if_none(args: dict, key: str, value: bool | int | str) -> None:
    if key not in args or args[key] is None:
        args[key] = value


def raise_file_handle_limit() -> None:
    """
    This is equivalent to the file handle limit raise that's done internally
    in Servo. The issue is that macOS uses a very low file handle limit (256)
    which is easily exhausted by the semaphore usage of WPT harness.
    """
    if sys.platform == "win32":
        return

    import resource

    try:
        FILE_HANDLE_LIMIT = 8192
        soft_limit, hard_limit = resource.getrlimit(resource.RLIMIT_NOFILE)
        # Don't change the value if we would be lowering it.
        if soft_limit > FILE_HANDLE_LIMIT:
            return
        if soft_limit == resource.RLIM_INFINITY:
            new_soft_limit = FILE_HANDLE_LIMIT
        else:
            new_soft_limit = min(hard_limit, FILE_HANDLE_LIMIT)
        resource.setrlimit(resource.RLIMIT_NOFILE, (new_soft_limit, hard_limit))
    except Exception as _:
        print("Could not raise the file handle limit")


def run_tests(default_binary_path: str, multiprocess: bool, **kwargs: Any) -> int:
    print(f"Running WPT tests with {default_binary_path}")

    os.environ["RUST_BACKTRACE"] = "1"
    os.environ["HOST_FILE"] = os.path.join(SERVO_ROOT, "tests", "wpt", "hosts")

    # The pytest framework used in the webdriver conformance tests dumps the
    # environment variables when unexpected results occur, and this variable
    # makes CI logs unreadable.
    os.environ.pop("GITHUB_CONTEXT", None)
    # Also remove this environment variable to prevent it leaking into the logs.
    gh_token = os.environ.pop("GH_TOKEN", None)

    # Allow to run with the legacy Servo WPT configuration. This is required
    # until necessary improvements are made to the debugging experience with
    # servodriver. See https://github.com/servo/servo/issues/40751
    product = "servo_legacy" if kwargs.get("legacy") else "servo"

    set_if_none(kwargs, "product", product)
    set_if_none(kwargs, "config", os.path.join(WPT_PATH, "config.ini"))
    set_if_none(kwargs, "include_manifest", os.path.join(WPT_PATH, "include.ini"))

    ensure_updated_manifest = kwargs.get("manifest_update") is not False
    kwargs["manifest_update"] = False

    raise_file_handle_limit()
    set_if_none(kwargs, "processes", multiprocessing.cpu_count())

    set_if_none(kwargs, "ca_cert_path", os.path.join(CERTS_PATH, "cacert.pem"))
    set_if_none(kwargs, "host_key_path", os.path.join(CERTS_PATH, "web-platform.test.key"))
    set_if_none(kwargs, "host_cert_path", os.path.join(CERTS_PATH, "web-platform.test.pem"))
    # Set `id_hash` as the default chunk, as this better distributes testing across different
    # chunks and leads to more consistent timing on GitHub Actions.
    set_if_none(kwargs, "chunk_type", "id_hash")

    set_if_none(kwargs, "binary", default_binary_path)
    set_if_none(kwargs, "webdriver_binary", default_binary_path)

    if kwargs.pop("rr_chaos"):
        kwargs["debugger"] = "rr"
        kwargs["debugger_args"] = "record --chaos"
        kwargs["repeat_until_unexpected"] = True
        # TODO: Delete rr traces from green test runs?

    prefs = kwargs.pop("prefs")
    if multiprocess:
        kwargs.setdefault("binary_args", ["-M"])

    given_http_proxy_uri = False
    given_https_proxy_uri = False
    if prefs:
        for pref in prefs:
            kwargs["binary_args"].append("--pref=" + pref)
            given_http_proxy_uri |= "network_http_proxy_uri" in pref
            given_https_proxy_uri |= "network_https_proxy_uri" in pref
    # We clearly dictates no proxy unless users know what they are doing.
    # This is to override potential default http_proxy/https_proxy.
    if not given_http_proxy_uri:
        kwargs["binary_args"].append("--pref=network_http_proxy_uri=")

    if not given_https_proxy_uri:
        kwargs["binary_args"].append("--pref=network_https_proxy_uri=")

    if not kwargs.get("no_default_test_types"):
        test_types = {
            "servo": ["testharness", "reftest", "wdspec", "crashtest"],
            "servo_legacy": ["testharness", "reftest", "wdspec", "crashtest"],
        }
        kwargs["test_types"] = test_types[product]

    filter_intermittents_output = kwargs.pop("filter_intermittents", None)
    stable_unexpected_raw_log_output_file = kwargs.pop("log_raw_stable_unexpected", None)
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
        if file_ext in [".htm", ".html", ".js", ".xhtml", ".xht", ".py", ".svg"]:
            use_mach_logging = True

    # Enable headless mode by default, unless `--no-headless` is explicitly passed, in
    # which case we do not run in headless mode.
    if kwargs.get("headless", None) is None:
        kwargs["headless"] = True

    if use_mach_logging:
        logger = wptrunner.setup_logging(kwargs, {"mach": sys.stdout})
    else:
        logger = wptrunner.setup_logging(kwargs, {"servo": sys.stdout})

    handler = ServoHandler(detect_flakes=kwargs["retry_unexpected"] >= 1)
    assert logger is not None
    logger.add_handler(handler)

    if ensure_updated_manifest:
        ensure_manifests_up_to_date(logger)

    with tempfile.TemporaryDirectory(prefix="servo-") as config_dir:
        kwargs["binary_args"] += ["--config-dir", config_dir]
        # Temporary workaround to avoid shared storage across parallel processes.
        # Can be removed once per-process config dirs are supported.
        kwargs["binary_args"] += ["--temporary-storage"]

        wptrunner.run_tests(**kwargs)

    return_value = int(handler.any_stable_unexpected())

    # Filter intermittents if that was specified on the command-line.
    if filter_intermittents_output:
        all_filtered = filter_intermittents(handler.unexpected_results, filter_intermittents_output, gh_token)
        return_value = 0 if all_filtered else 1

    # Write the unexpected-only raw log if that was specified on the command-line.
    if stable_unexpected_raw_log_output_file:
        if not raw_log_outputs:
            print("'--log-raw-stable-unexpected' not written without '--log-raw'.")
        else:
            write_stable_unexpected_only_raw_log(
                handler.unexpected_results, raw_log_outputs[0].name, stable_unexpected_raw_log_output_file
            )

    return return_value


def get_intermittent_issues_from_github(gh_token: str | None) -> list[dict]:
    environment = dict(os.environ)
    if gh_token:
        environment["GH_TOKEN"] = gh_token

    try:
        output = subprocess.check_output(
            ["gh", "api", "graphql", "--jq", ".data.search.nodes[]", "-f", f"query={INTERMITTENT_ISSUES_GRAPHQL}"],
            env=environment,
            text=True,
        )
    except (FileNotFoundError, subprocess.CalledProcessError):
        print("Filtering intermittents requires having `gh` installed and authenticated.")
        return []
    return [json.loads(line) for line in output.splitlines()]


def filter_intermittents(unexpected_results: List[UnexpectedResult], output_path: str, gh_token: str | None) -> bool:
    print(f"Filtering {len(unexpected_results)} unexpected results for known intermittents")
    known_intermittents = get_intermittent_issues_from_github(gh_token)
    if known_intermittents:
        for result in unexpected_results:
            result.issues = [
                intermittent["number"] for intermittent in known_intermittents if result.path in intermittent["title"]
            ]

    def add_result(
        output: list[str],
        text: str,
        results: List[UnexpectedResult],
        filter_func: Callable[[UnexpectedResult], bool],
    ) -> None:
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
        lambda result: not result.flaky and bool(result.issues),
    )
    add_result(output, "Stable unexpected results", unexpected_results, is_stable_and_unexpected)
    print("\n".join(output))

    with open(output_path, "w", encoding="utf-8") as file:
        json.dump([dataclasses.asdict(result) for result in unexpected_results], file)

    return not any([is_stable_and_unexpected(result) for result in unexpected_results])


def write_stable_unexpected_only_raw_log(
    unexpected_results: List[UnexpectedResult], raw_log_file: str, filtered_raw_log_file: str
) -> None:
    # Only write the data for tests which are not flaky and which do not have issues.
    # This allows the resulting log file to be used to update baselines after a CI run,
    # as it will only contain stable unexpected results without issues.
    tests = [result.path for result in unexpected_results if not result.flaky and not result.issues]
    print(f"Writing stable unexpected-only raw log to {filtered_raw_log_file}")

    with open(filtered_raw_log_file, "w", encoding="utf-8") as output:
        with open(raw_log_file) as input:
            for line in input.readlines():
                data = json.loads(line)
                if data["action"] in ["suite_start", "suite_end", "add_subsuite"] or (
                    "test" in data and data["test"] in tests
                ):
                    output.write(line)
