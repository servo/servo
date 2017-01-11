#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import argparse
import itertools
import json
import os
import subprocess
from functools import partial
from statistics import median, StatisticsError


def load_manifest(filename):
    with open(filename, 'r') as f:
        text = f.read()
    return list(parse_manifest(text))


def parse_manifest(text):
    lines = filter(lambda x: x != "" and not x.startswith("#"),
                   map(lambda x: x.strip(), text.splitlines()))
    output = []
    for line in lines:
        if line.split(" ")[0] == "async":
            output.append((line.split(" ")[1], True))
        else:
            output.append((line.split(" ")[0], False))
    return output


def execute_test(url, command, timeout):
    try:
        return subprocess.check_output(
            command, stderr=subprocess.STDOUT, timeout=timeout
        )
    except subprocess.CalledProcessError as e:
        print("Unexpected Fail:")
        print(e)
        print("You may want to re-run the test manually:\n{}"
              .format(' '.join(command)))
    except subprocess.TimeoutExpired:
        print("Test FAILED due to timeout: {}".format(url))
    return ""


def run_servo_test(url, timeout, is_async):
    if is_async:
        print("Servo does not support async test!")
        # Return a placeholder
        return parse_log("", url)

    ua_script_path = "{}/user-agent-js".format(os.getcwd())
    command = [
        "../../../target/release/servo", url,
        "--userscripts", ua_script_path,
        "--headless",
        "-x", "-o", "output.png"
    ]
    log = ""
    try:
        log = subprocess.check_output(
            command, stderr=subprocess.STDOUT, timeout=timeout
        )
    except subprocess.CalledProcessError as e:
        print("Unexpected Fail:")
        print(e)
        print("You may want to re-run the test manually:\n{}".format(
            ' '.join(command)
        ))
    except subprocess.TimeoutExpired:
        print("Test FAILED due to timeout: {}".format(url))
    return parse_log(log, url)


def parse_log(log, testcase):
    blocks = []
    block = []
    copy = False
    for line_bytes in log.splitlines():
        line = line_bytes.decode()

        if line.strip() == ("[PERF] perf block start"):
            copy = True
        elif line.strip() == ("[PERF] perf block end"):
            copy = False
            blocks.append(block)
            block = []
        elif copy and line.strip().startswith("[PERF]"):
            block.append(line)

    def parse_block(block):
        timing = {}
        for line in block:
            try:
                (_, key, value) = line.split(",")
            except:
                print("[DEBUG] failed to parse the following line:")
                print(line)
                print('[DEBUG] log:')
                print('-----')
                print(log)
                print('-----')
                return None

            if key == "testcase" or key == "title":
                timing[key] = value
            else:
                timing[key] = None if (value == "undefined") else int(value)

        return timing

    def valid_timing(timing, testcase=None):
        if (timing is None or
                testcase is None or
                timing.get('title') == 'Error response' or
                timing.get('testcase') != testcase):
            return False
        else:
            return True

    # We need to still include the failed tests, otherwise Treeherder will
    # consider the result to be a new test series, and thus a new graph. So we
    # use a placeholder with values = -1 to make Treeherder happy, and still be
    # able to identify failed tests (successful tests have time >=0).
    def create_placeholder(testcase):
        return {
            "testcase": testcase,
            "title": "",
            "navigationStart": 0,
            "unloadEventStart": -1,
            "domLoading": -1,
            "fetchStart": -1,
            "responseStart": -1,
            "loadEventEnd": -1,
            "connectStart": -1,
            "domainLookupStart": -1,
            "redirectStart": -1,
            "domContentLoadedEventEnd": -1,
            "requestStart": -1,
            "secureConnectionStart": -1,
            "connectEnd": -1,
            "loadEventStart": -1,
            "domInteractive": -1,
            "domContentLoadedEventStart": -1,
            "redirectEnd": -1,
            "domainLookupEnd": -1,
            "unloadEventEnd": -1,
            "responseEnd": -1,
            "domComplete": -1,
        }

    valid_timing_for_case = partial(valid_timing, testcase=testcase)
    timings = list(filter(valid_timing_for_case, map(parse_block, blocks)))

    if len(timings) == 0:
        print("Didn't find any perf data in the log, test timeout?")
        print('[DEBUG] log:')
        print('-----')
        print(log)
        print('-----')

        return [create_placeholder(testcase)]
    else:
        return timings


def filter_result_by_manifest(result_json, manifest):
    filtered = []
    for name, is_async in manifest:
        match = [tc for tc in result_json if tc['testcase'] == name]
        if len(match) == 0:
            raise Exception(("Missing test result: {}. This will cause a "
                             "discontinuity in the treeherder graph, "
                             "so we won't submit this data.").format(name))
        filtered += match
    return filtered


def take_result_median(result_json, expected_runs):
    median_results = []
    for k, g in itertools.groupby(result_json, lambda x: x['testcase']):
        group = list(g)
        if len(group) != expected_runs:
            print(("Warning: Not enough test data for {},"
                  " maybe some runs failed?").format(k))

        median_result = {}
        for k, _ in group[0].items():
            if k == "testcase" or k == "title":
                median_result[k] = group[0][k]
            else:
                try:
                    median_result[k] = median([x[k] for x in group
                                               if x[k] is not None])
                except StatisticsError:
                    median_result[k] = -1
        median_results.append(median_result)
    return median_results


def save_result_json(results, filename, manifest, expected_runs):

    results = filter_result_by_manifest(results, manifest)
    results = take_result_median(results, expected_runs)

    if len(results) == 0:
        with open(filename, 'w') as f:
            json.dump("No test result found in the log. All tests timeout?",
                      f, indent=2)
    else:
        with open(filename, 'w') as f:
            json.dump(results, f, indent=2)
    print("Result saved to {}".format(filename))


def format_result_summary(results):
    failures = list(filter(lambda x: x['domComplete'] == -1, results))
    result_log = """
========================================
Total {total} tests; {suc} succeeded, {fail} failed.

Failure summary:
""".format(
        total=len(results),
        suc=len(list(filter(lambda x: x['domComplete'] != -1, results))),
        fail=len(failures)
    )
    uniq_failures = list(set(map(lambda x: x['testcase'], failures)))
    for failure in uniq_failures:
        result_log += " - {}\n".format(failure)

    result_log += "========================================\n"

    return result_log


def main():
    parser = argparse.ArgumentParser(
        description="Run page load test on servo"
    )
    parser.add_argument("tp5_manifest",
                        help="the test manifest in tp5 format")
    parser.add_argument("output_file",
                        help="filename for the output json")
    parser.add_argument("--runs",
                        type=int,
                        default=20,
                        help="number of runs for each test case. Defult: 20")
    parser.add_argument("--timeout",
                        type=int,
                        default=300,  # 5 min
                        help=("kill the test if not finished in time (sec)."
                              " Default: 5 min"))
    parser.add_argument("--engine",
                        type=str,
                        default='servo',
                        help=("The engine to run the tests on. Currently only"
                              " servo and gecko are supported."))
    args = parser.parse_args()
    if args.engine == 'servo':
        run_test = run_servo_test
    elif args.engine == 'gecko':
        import gecko_driver  # Load this only when we need gecko test
        run_test = gecko_driver.run_gecko_test
    try:
        # Assume the server is up and running
        testcases = load_manifest(args.tp5_manifest)
        results = []
        for testcase, is_async in testcases:
            for run in range(args.runs):
                print("Running test {}/{} on {}".format(run + 1,
                                                        args.runs,
                                                        testcase))
                # results will be a mixure of timings dict and testcase strings
                # testcase string indicates a failed test
                results += run_test(testcase, args.timeout, is_async)
                print("Finished")
                # TODO: Record and analyze other performance.timing properties

        print(format_result_summary(results))
        save_result_json(results, args.output_file, testcases, args.runs)

    except KeyboardInterrupt:
        print("Test stopped by user, saving partial result")
        save_result_json(results, args.output_file, testcases, args.runs)


if __name__ == "__main__":
    main()
