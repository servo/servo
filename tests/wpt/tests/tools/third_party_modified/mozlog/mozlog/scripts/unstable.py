# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import argparse
import json
from collections import defaultdict

from mozlog import reader


class StatusHandler(reader.LogHandler):
    def __init__(self):
        self.run_info = None
        self.statuses = defaultdict(
            lambda: defaultdict(lambda: defaultdict(lambda: defaultdict(int)))
        )

    def test_id(self, test):
        if isinstance(test, str):
            return test
        else:
            return tuple(test)

    def suite_start(self, item):
        self.run_info = tuple(sorted(item.get("run_info", {}).items()))

    def test_status(self, item):
        self.statuses[self.run_info][self.test_id(item["test"])][item["subtest"]][
            item["status"]
        ] += 1

    def test_end(self, item):
        self.statuses[self.run_info][self.test_id(item["test"])][None][
            item["status"]
        ] += 1

    def suite_end(self, item):
        self.run_info = None


def get_statuses(filenames):
    handler = StatusHandler()

    for filename in filenames:
        with open(filename) as f:
            reader.handle_log(reader.read(f), handler)

    return handler.statuses


def _filter(results_cmp):
    def inner(statuses):
        rv = defaultdict(lambda: defaultdict(dict))

        for run_info, tests in statuses.items():
            for test, subtests in tests.items():
                for name, results in subtests.items():
                    if results_cmp(results):
                        rv[run_info][test][name] = results

        return rv

    return inner


filter_unstable = _filter(lambda x: len(x) > 1)
filter_stable = _filter(lambda x: len(x) == 1)


def group_results(data):
    rv = defaultdict(lambda: defaultdict(lambda: defaultdict(int)))

    for run_info, tests in data.items():
        for test, subtests in tests.items():
            for name, results in subtests.items():
                for status, number in results.items():
                    rv[test][name][status] += number
    return rv


def print_results(data):
    for run_info, tests in data.items():
        run_str = (
            " ".join("%s:%s" % (k, v) for k, v in run_info)
            if run_info
            else "No Run Info"
        )
        print(run_str)
        print("=" * len(run_str))
        print_run(tests)


def print_run(tests):
    for test, subtests in sorted(tests.items()):
        print("\n" + str(test))
        print("-" * len(test))
        for name, results in subtests.items():
            print(
                "[%s]: %s"
                % (
                    name if name is not None else "",
                    " ".join("%s (%i)" % (k, v) for k, v in results.items()),
                )
            )


def get_parser(add_help=True):
    parser = argparse.ArgumentParser(
        "unstable",
        description="List tests that don't give consistent "
        "results from one or more runs.",
        add_help=add_help,
    )
    parser.add_argument(
        "--json", action="store_true", default=False, help="Output in JSON format"
    )
    parser.add_argument(
        "--group",
        action="store_true",
        default=False,
        help="Group results from different run types",
    )
    parser.add_argument("log_file", nargs="+", help="Log files to read")
    return parser


def main(**kwargs):
    unstable = filter_unstable(get_statuses(kwargs["log_file"]))
    if kwargs["group"]:
        unstable = group_results(unstable)

    if kwargs["json"]:
        print(json.dumps(unstable))
    else:
        if not kwargs["group"]:
            print_results(unstable)
        else:
            print_run(unstable)


if __name__ == "__main__":
    parser = get_parser()
    args = parser.parse_args()
    kwargs = vars(args)
    main(**kwargs)
