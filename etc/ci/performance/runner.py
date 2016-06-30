#!/usr/bin/python3
import argparse
import itertools
import json
import os
import subprocess
from statistics import median, StatisticsError


def load_manifest(filename):
    with open(filename, 'r') as f:
        text = f.read()
    return list(parse_manifest(text))


def parse_manifest(text):
    return filter(lambda x: x != "" and not x.startswith("#"),
                  map(lambda x: x.strip(), text.splitlines()))


def execute_test(url, command, timeout):
    print("Running test:")
    print(command)
    print("Timeout:{}".format(timeout))
    try:
        return subprocess.check_output(command, stderr=subprocess.STDOUT,
                                       shell=True, timeout=timeout)
    except subprocess.CalledProcessError as e:
        print("Unexpected Fail:")
        print(e)
        print("You may want to re-run the test manually:\n{}".format(command))
    except subprocess.TimeoutExpired:
        print("Test timeout: {}".format(url))
    return ""


def get_servo_command(url, timeout):
    ua_script_path = "{}/user-agent-js".format(os.getcwd())
    test_cmd = ("timeout {timeout}s ./servo/servo '{url}'"
                " --userscripts {ua} -x -o {png}").format(timeout=timeout,
                                                          url=url,
                                                          ua=ua_script_path,
                                                          png="output.png")
    return test_cmd


def get_gecko_command(url, timeout):
    test_cmd = ("timeout {timeout}s firefox --no-remote --profile ./firefox/servo {url}"
                .format(timeout=timeout, url=url))
    return test_cmd


def parse_log(log, testcase=None):
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
        elif copy:
            block.append(line)

    def parse_block(block):
        timing = {}
        for line in block:
            key = line.split(",")[1]
            value = line.split(",")[2]

            if key == "testcase":
                timing[key] = value
            else:
                timing[key] = None if (value == "undefined") else int(value)
        return timing

    if len(blocks) == 0:
        print("Didn't find any perf data in the log, test timeout?")
        print("Fillng in a dummy perf data")
        return [{
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
            "testcase": testcase,
            "domComplete": -1,
        }]
    else:
        return map(parse_block, blocks)


def filter_result_by_manifest(result_json, manifest):
    # print(manifest)
    # print(result_json)
    return [tc for tc in result_json if tc['testcase'] in manifest]


def take_result_median(result_json, expected_runs):
    median_results = []
    for k, g in itertools.groupby(result_json, lambda x: x['testcase']):
        group = list(g)
        if len(group) != expected_runs:
            print(("Warning: Not enough test data for {},"
                  " maybe some runs failed?").format(k))

        median_result = {}
        for k, _ in group[0].items():
            if k == "testcase":
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

    # print(results)
    results = filter_result_by_manifest(results, manifest)
    # print(results)
    results = take_result_median(results, expected_runs)
    # print(results)

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
            suc =len(list(filter(lambda x: x['domComplete'] != -1, results))),
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
        command_factory = get_servo_command
    elif args.engine == 'gecko':
        command_factory = get_gecko_command
    try:
        # Assume the server is up and running
        testcases = load_manifest(args.tp5_manifest)
        results = []
        for testcase in testcases:
            for run in range(args.runs):
                print("Running test {}/{} on {}".format(run + 1,
                                                        args.runs,
                                                        testcase))
                command = command_factory(testcase, args.timeout)
                log = execute_test(testcase, command, args.timeout)
                result = parse_log(log, testcase)
                # TODO: check for other measurements
                results += result

        print(format_result_summary(results))
        save_result_json(results, args.output_file, testcases, args.runs)

    except KeyboardInterrupt:
        print("Test stopped by user, saving partial result")
        save_result_json(results, args.output_file, testcases, args.runs)


if __name__ == "__main__":
    main()
