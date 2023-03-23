# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import dataclasses
import grouping_formatter
import json
import os
import re
import sys
import urllib.error
import urllib.parse
import urllib.request

import mozlog
import mozlog.formatters
import multiprocessing

from typing import List, NamedTuple, Optional, Union
from grouping_formatter import UnexpectedResult, UnexpectedSubtestResult

SCRIPT_PATH = os.path.abspath(os.path.dirname(__file__))
SERVO_ROOT = os.path.abspath(os.path.join(SCRIPT_PATH, "..", ".."))
WPT_TOOLS_PATH = os.path.join(SCRIPT_PATH, "web-platform-tests", "tools")
CERTS_PATH = os.path.join(WPT_TOOLS_PATH, "certs")

sys.path.insert(0, WPT_TOOLS_PATH)
import localpaths  # noqa: F401,E402
import update  # noqa: F401,E402

TRACKER_API = "https://build.servo.org/intermittent-tracker"
TRACKER_API_ENV_VAR = "INTERMITTENT_TRACKER_API"
TRACKER_DASHBOARD_SECRET_ENV_VAR = "INTERMITTENT_TRACKER_DASHBOARD_SECRET"


def determine_build_type(kwargs: dict, target_dir: str):
    if kwargs["release"]:
        return "release"
    elif kwargs["debug"]:
        return "debug"
    elif os.path.exists(os.path.join(target_dir, "debug")):
        return "debug"
    elif os.path.exists(os.path.join(target_dir, "release")):
        return "release"
    return "debug"


def set_if_none(args: dict, key: str, value):
    if key not in args or args[key] is None:
        args[key] = value


def update_args_for_layout_2020(kwargs: dict):
    if kwargs.pop("layout_2020"):
        kwargs["test_paths"]["/"]["metadata_path"] = os.path.join(
            SCRIPT_PATH, "metadata-layout-2020"
        )
        kwargs["test_paths"]["/_mozilla/"]["metadata_path"] = os.path.join(
            SCRIPT_PATH, "mozilla", "meta-layout-2020"
        )
        kwargs["include_manifest"] = os.path.join(
            SCRIPT_PATH, "include-layout-2020.ini"
        )


def run_tests(**kwargs):
    from wptrunner import wptrunner
    from wptrunner import wptcommandline

    # By default, Rayon selects the number of worker threads based on the
    # available CPU count. This doesn't work very well when running tests on CI,
    # since we run so many Servo processes in parallel. The result is a lot of
    # extra timeouts. Instead, force Rayon to assume we are running on a 2 CPU
    # environment.
    os.environ["RAYON_RS_NUM_CPUS"] = "2"
    os.environ["RUST_BACKTRACE"] = "1"
    os.environ["HOST_FILE"] = os.path.join(SERVO_ROOT, "tests", "wpt", "hosts")

    set_if_none(kwargs, "product", "servo")
    set_if_none(kwargs, "config", os.path.join(SCRIPT_PATH, "config.ini"))
    set_if_none(kwargs, "include_manifest", os.path.join(SCRIPT_PATH, "include.ini"))
    set_if_none(kwargs, "manifest_update", False)
    set_if_none(kwargs, "processes", multiprocessing.cpu_count())

    set_if_none(kwargs, "ca_cert_path", os.path.join(CERTS_PATH, "cacert.pem"))
    set_if_none(
        kwargs, "host_key_path", os.path.join(CERTS_PATH, "web-platform.test.key")
    )
    set_if_none(
        kwargs, "host_cert_path", os.path.join(CERTS_PATH, "web-platform.test.pem")
    )

    kwargs["user_stylesheets"].append(os.path.join(SERVO_ROOT, "resources", "ahem.css"))

    if "CARGO_TARGET_DIR" in os.environ:
        target_dir = os.path.join(os.environ["CARGO_TARGET_DIR"])
    else:
        target_dir = os.path.join(SERVO_ROOT, "target")
    default_binary_path = os.path.join(
        target_dir, determine_build_type(kwargs, target_dir), "servo"
    )
    if sys.platform == "win32":
        target_dir += ".exe"

    set_if_none(kwargs, "binary", default_binary_path)
    set_if_none(kwargs, "webdriver_binary", default_binary_path)

    if kwargs.pop("rr_chaos"):
        kwargs["debugger"] = "rr"
        kwargs["debugger_args"] = "record --chaos"
        kwargs["repeat_until_unexpected"] = True
        # TODO: Delete rr traces from green test runs?

    prefs = kwargs.pop("prefs")
    if prefs:
        kwargs["binary_args"] = ["--pref=" + pref for pref in prefs]

    if not kwargs.get("no_default_test_types"):
        test_types = {
            "servo": ["testharness", "reftest", "wdspec"],
            "servodriver": ["testharness", "reftest"],
        }
        product = kwargs.get("product") or "servo"
        kwargs["test_types"] = test_types[product]

    filter_intermittents_output = kwargs.pop("filter_intermittents", None)
    unexpected_raw_log_output_file = kwargs.pop("log_raw_unexpected", None)
    raw_log_outputs = kwargs.get("log_raw", [])

    wptcommandline.check_args(kwargs)
    update_args_for_layout_2020(kwargs)

    mozlog.commandline.log_formatters["servo"] = (
        grouping_formatter.ServoFormatter,
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

    handler = grouping_formatter.ServoHandler()
    logger.add_handler(handler)

    wptrunner.run_tests(**kwargs)
    return_value = 0 if not handler.unexpected_results else 1

    # Filter intermittents if that was specified on the command-line.
    if handler.unexpected_results and filter_intermittents_output:
        # Copy the list of unexpected results from the first run, so that we
        # can access them after the tests are rerun (which changes
        # `handler.unexpected_results`). After rerunning some tests will be
        # marked as flaky but otherwise the contents of this original list
        # won't change.
        unexpected_results = list(handler.unexpected_results)

        # This isn't strictly necessary since `handler.suite_start()` clears
        # the state, but make sure that we are starting with a fresh handler.
        handler.reset_state()

        print(80 * "=")
        print(f"Rerunning {len(unexpected_results)} tests "
              "with unexpected results to detect flaky tests.")
        unexpected_results_tests = [result.path for result in unexpected_results]
        kwargs["test_list"] = unexpected_results_tests
        kwargs["include"] = unexpected_results_tests
        kwargs["pause_after_test"] = False
        wptrunner.run_tests(**kwargs)

        # Use the second run to mark tests from the first run as flaky, but
        # discard the results otherwise.
        # TODO: It might be a good idea to send the new results to the
        # dashboard if they were also unexpected.
        stable_tests = [result.path for result in handler.unexpected_results]
        for result in unexpected_results:
            result.flaky = result.path not in stable_tests

        all_filtered = filter_intermittents(unexpected_results,
                                            filter_intermittents_output)
        return_value = 0 if all_filtered else 1

    # Write the unexpected-only raw log if that was specified on the command-line.
    if unexpected_raw_log_output_file:
        if not raw_log_outputs:
            print("'--log-raw-unexpected' not written without '--log-raw'.")
        else:
            write_unexpected_only_raw_log(
                handler.unexpected_results,
                raw_log_outputs[0].name,
                unexpected_raw_log_output_file
            )

    return return_value


def update_tests(**kwargs):
    from update import updatecommandline

    set_if_none(kwargs, "product", "servo")
    set_if_none(kwargs, "config", os.path.join(SCRIPT_PATH, "config.ini"))
    kwargs["store_state"] = False

    updatecommandline.check_args(kwargs)
    update_args_for_layout_2020(kwargs)

    logger = update.setup_logging(kwargs, {"mach": sys.stdout})
    return_value = update.run_update(logger, **kwargs)
    return 1 if return_value is update.exit_unclean else 0


class GithubContextInformation(NamedTuple):
    build_url: Optional[str]
    pull_url: Optional[str]
    branch_name: Optional[str]


class TrackerDashboardFilter():
    def __init__(self):
        base_url = os.environ.get(TRACKER_API_ENV_VAR, TRACKER_API)
        self.headers = {
            "Content-Type": "application/json"
        }
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

        repository = github_context['repository']
        repo_url = f"https://github.com/{repository}"

        run_id = github_context['run_id']
        build_url = f"{repo_url}/actions/runs/{run_id}"

        commit_title = github_context["event"]["head_commit"]["message"]
        match = re.match(r"^Auto merge of #(\d+)", commit_title)
        pr_url = f"{repo_url}/pull/{match.group(1)}" if match else None

        return GithubContextInformation(
            build_url,
            pr_url,
            github_context["ref_name"]
        )

    def make_data_from_result(
        self,
        result: Union[UnexpectedResult, UnexpectedSubtestResult],
    ) -> dict:
        data = {
            'path': result.path,
            'subtest': None,
            'expected': result.expected,
            'actual': result.actual,
            'time': result.time // 1000,
            'message': result.message,
            'stack': result.stack,
        }
        if isinstance(result, UnexpectedSubtestResult):
            data["subtest"] = result.subtest
        return data

    def report_failures(self, unexpected_results: List[UnexpectedResult]):
        attempts = []
        for result in unexpected_results:
            attempts.append(self.make_data_from_result(result))
            for subtest_result in result.unexpected_subtest_results:
                attempts.append(self.make_data_from_result(subtest_result))

        context = self.get_github_context_information()
        try:
            request = urllib.request.Request(
                url=self.url,
                method='POST',
                data=json.dumps({
                    'branch': context.branch_name,
                    'build_url': context.build_url,
                    'pull_url': context.pull_url,
                    'attempts': attempts
                }).encode('utf-8'),
                headers=self.headers)

            known_intermittents = dict()
            with urllib.request.urlopen(request) as response:
                for test in json.load(response)["known"]:
                    known_intermittents[test["path"]] = \
                        [issue["number"] for issue in test["issues"]]

        except urllib.error.HTTPError as e:
            print(e)
            print(e.readlines())
            raise(e)

        for result in unexpected_results:
            result.issues = known_intermittents.get(result.path, [])


def filter_intermittents(
    unexpected_results: List[UnexpectedResult],
    output_path: str
) -> bool:
    print(f"Filtering {len(unexpected_results)} "
          "unexpected results for known intermittents")
    dashboard = TrackerDashboardFilter()
    dashboard.report_failures(unexpected_results)

    def add_result(output, text, results, filter_func) -> None:
        filtered = [str(result) for result in filter(filter_func, results)]
        if filtered:
            output += [f"{text} ({len(results)}): ", *filtered]

    def is_stable_and_unexpected(result):
        return not result.flaky and not result.issues

    output = []
    add_result(output, "Flaky unexpected results", unexpected_results,
               lambda result: result.flaky)
    add_result(output, "Stable unexpected results that are known-intermittent",
               unexpected_results, lambda result: not result.flaky and result.issues)
    add_result(output, "Stable unexpected results",
               unexpected_results, is_stable_and_unexpected)
    print("\n".join(output))

    with open(output_path, "w", encoding="utf-8") as file:
        json.dump([dataclasses.asdict(result) for result in unexpected_results], file)

    return not any([is_stable_and_unexpected(result) for result in unexpected_results])


def write_unexpected_only_raw_log(
    unexpected_results: List[UnexpectedResult],
    raw_log_file: str,
    filtered_raw_log_file: str
):
    tests = [result.path for result in unexpected_results]
    print(f"Writing unexpected-only raw log to {filtered_raw_log_file}")

    with open(filtered_raw_log_file, "w", encoding="utf-8") as output:
        with open(raw_log_file) as input:
            for line in input.readlines():
                data = json.loads(line)
                if data["action"] in ["suite_start", "suite_end"] or \
                        ("test" in data and data["test"] in tests):
                    output.write(line)


def main():
    from wptrunner import wptcommandline

    parser = wptcommandline.create_parser()
    kwargs = vars(parser.parse_args())
    return run_tests(**kwargs)


if __name__ == "__main__":
    sys.exit(0 if main() else 1)
