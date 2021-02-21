import copy
import functools
import imp
import io
import os
from collections import OrderedDict, defaultdict
from datetime import datetime

from mozlog import reader
from mozlog.formatters import JSONFormatter
from mozlog.handlers import BaseHandler, StreamHandler, LogLevelFilter

here = os.path.dirname(__file__)
localpaths = imp.load_source("localpaths", os.path.abspath(os.path.join(here, os.pardir, os.pardir, "localpaths.py")))
from ci.tc.github_checks_output import get_gh_checks_outputter
from wpt.markdown import markdown_adjust, table


# If a test takes more than (FLAKY_THRESHOLD*timeout) and does not consistently
# time out, it is considered slow (potentially flaky).
FLAKY_THRESHOLD = 0.8


class LogActionFilter(BaseHandler):

    """Handler that filters out messages not of a given set of actions.

    Subclasses BaseHandler.

    :param inner: Handler to use for messages that pass this filter
    :param actions: List of actions for which to fire the handler
    """

    def __init__(self, inner, actions):
        """Extend BaseHandler and set inner and actions props on self."""
        BaseHandler.__init__(self, inner)
        self.inner = inner
        self.actions = actions

    def __call__(self, item):
        """Invoke handler if action is in list passed as constructor param."""
        if item["action"] in self.actions:
            return self.inner(item)


class LogHandler(reader.LogHandler):

    """Handle updating test and subtest status in log.

    Subclasses reader.LogHandler.
    """
    def __init__(self):
        self.results = OrderedDict()

    def find_or_create_test(self, data):
        test_name = data["test"]
        if self.results.get(test_name):
            return self.results[test_name]

        test = {
            "subtests": OrderedDict(),
            "status": defaultdict(int),
            "longest_duration": defaultdict(float),
        }
        self.results[test_name] = test
        return test

    def find_or_create_subtest(self, data):
        test = self.find_or_create_test(data)
        subtest_name = data["subtest"]

        if test["subtests"].get(subtest_name):
            return test["subtests"][subtest_name]

        subtest = {
            "status": defaultdict(int),
            "messages": set()
        }
        test["subtests"][subtest_name] = subtest

        return subtest

    def test_start(self, data):
        test = self.find_or_create_test(data)
        test["start_time"] = data["time"]

    def test_status(self, data):
        subtest = self.find_or_create_subtest(data)
        subtest["status"][data["status"]] += 1
        if data.get("message"):
            subtest["messages"].add(data["message"])

    def test_end(self, data):
        test = self.find_or_create_test(data)
        test["status"][data["status"]] += 1
        # Timestamps are in ms since epoch.
        duration = data["time"] - test.pop("start_time")
        test["longest_duration"][data["status"]] = max(
            duration, test["longest_duration"][data["status"]])
        try:
            # test_timeout is in seconds; convert it to ms.
            test["timeout"] = data["extra"]["test_timeout"] * 1000
        except KeyError:
            # If a test is skipped, it won't have extra info.
            pass


def is_inconsistent(results_dict, iterations):
    """Return whether or not a single test is inconsistent."""
    if 'SKIP' in results_dict:
        return False
    return len(results_dict) > 1 or sum(results_dict.values()) != iterations


def find_slow_status(test):
    """Check if a single test almost times out.

    We are interested in tests that almost time out (i.e. likely to be flaky).
    Therefore, timeout statuses are ignored, including (EXTERNAL-)TIMEOUT.
    CRASH & ERROR are also ignored because the they override TIMEOUT; a test
    that both crashes and times out is marked as CRASH, so it won't be flaky.

    Returns:
        A result status produced by a run that almost times out; None, if no
        runs almost time out.
    """
    if "timeout" not in test:
        return None
    threshold = test["timeout"] * FLAKY_THRESHOLD
    for status in ['PASS', 'FAIL', 'OK']:
        if (status in test["longest_duration"] and
            test["longest_duration"][status] > threshold):
            return status
    return None


def process_results(log, iterations):
    """Process test log and return overall results and list of inconsistent tests."""
    inconsistent = []
    slow = []
    handler = LogHandler()
    reader.handle_log(reader.read(log), handler)
    results = handler.results
    for test_name, test in results.items():
        if is_inconsistent(test["status"], iterations):
            inconsistent.append((test_name, None, test["status"], []))
        for subtest_name, subtest in test["subtests"].items():
            if is_inconsistent(subtest["status"], iterations):
                inconsistent.append((test_name, subtest_name, subtest["status"], subtest["messages"]))

        slow_status = find_slow_status(test)
        if slow_status is not None:
            slow.append((
                test_name,
                slow_status,
                test["longest_duration"][slow_status],
                test["timeout"]
            ))

    return results, inconsistent, slow


def err_string(results_dict, iterations):
    """Create and return string with errors from test run."""
    rv = []
    total_results = sum(results_dict.values())
    if total_results > iterations:
        rv.append("Duplicate subtest name")
    else:
        for key, value in sorted(results_dict.items()):
            rv.append("%s%s" %
                      (key, ": %s/%s" % (value, iterations) if value != iterations else ""))
    if total_results < iterations:
        rv.append("MISSING: %s/%s" % (iterations - total_results, iterations))
    rv = ", ".join(rv)
    if is_inconsistent(results_dict, iterations):
        rv = "**%s**" % rv
    return rv


def write_github_checks_summary_inconsistent(log, inconsistent, iterations):
    """Outputs a summary of inconsistent tests for GitHub Checks."""
    log("Some affected tests had inconsistent (flaky) results:\n")
    write_inconsistent(log, inconsistent, iterations)
    log("\n")
    log("These may be pre-existing or new flakes. Please try to reproduce (see "
        "the above WPT command, though some flags may not be needed when "
        "running locally) and determine if your change introduced the flake. "
        "If you are unable to reproduce the problem, please tag "
        "`@web-platform-tests/wpt-core-team` in a comment for help.\n")


def write_github_checks_summary_slow_tests(log, slow):
    """Outputs a summary of slow tests for GitHub Checks."""
    log("Some affected tests had slow results:\n")
    write_slow_tests(log, slow)
    log("\n")
    log("These may be pre-existing or newly slow tests. Slow tests indicate "
        "that a test ran very close to the test timeout limit and so may "
        "become TIMEOUT-flaky in the future. Consider speeding up the test or "
        "breaking it into multiple tests. For help, please tag "
        "`@web-platform-tests/wpt-core-team` in a comment.\n")


def write_inconsistent(log, inconsistent, iterations):
    """Output inconsistent tests to the passed in logging function."""
    log("## Unstable results ##\n")
    strings = [(
        "`%s`" % markdown_adjust(test),
        ("`%s`" % markdown_adjust(subtest)) if subtest else "",
        err_string(results, iterations),
        ("`%s`" % markdown_adjust(";".join(messages))) if len(messages) else "")
        for test, subtest, results, messages in inconsistent]
    table(["Test", "Subtest", "Results", "Messages"], strings, log)


def write_slow_tests(log, slow):
    """Output slow tests to the passed in logging function."""
    log("## Slow tests ##\n")
    strings = [(
        "`%s`" % markdown_adjust(test),
        "`%s`" % status,
        "`%.0f`" % duration,
        "`%.0f`" % timeout)
        for test, status, duration, timeout in slow]
    table(["Test", "Result", "Longest duration (ms)", "Timeout (ms)"], strings, log)


def write_results(log, results, iterations, pr_number=None, use_details=False):
    log("## All results ##\n")
    if use_details:
        log("<details>\n")
        log("<summary>%i %s ran</summary>\n\n" % (len(results),
                                                  "tests" if len(results) > 1
                                                  else "test"))

    for test_name, test in results.items():
        baseurl = "http://w3c-test.org/submissions"
        if "https" in os.path.splitext(test_name)[0].split(".")[1:]:
            baseurl = "https://w3c-test.org/submissions"
        title = test_name
        if use_details:
            log("<details>\n")
            if pr_number:
                title = "<a href=\"%s/%s%s\">%s</a>" % (baseurl, pr_number, test_name, title)
            log('<summary>%s</summary>\n\n' % title)
        else:
            log("### %s ###" % title)
        strings = [("", err_string(test["status"], iterations), "")]

        strings.extend(((
            ("`%s`" % markdown_adjust(subtest_name)) if subtest else "",
            err_string(subtest["status"], iterations),
            ("`%s`" % markdown_adjust(';'.join(subtest["messages"]))) if len(subtest["messages"]) else "")
            for subtest_name, subtest in test["subtests"].items()))
        table(["Subtest", "Results", "Messages"], strings, log)
        if use_details:
            log("</details>\n")

    if use_details:
        log("</details>\n")


def run_step(logger, iterations, restart_after_iteration, kwargs_extras, **kwargs):
    from . import wptrunner
    kwargs = copy.deepcopy(kwargs)

    if restart_after_iteration:
        kwargs["repeat"] = iterations
    else:
        kwargs["rerun"] = iterations

    kwargs["pause_after_test"] = False
    kwargs.update(kwargs_extras)

    def wrap_handler(x):
        if not kwargs["verify_log_full"]:
            x = LogLevelFilter(x, "WARNING")
            x = LogActionFilter(x, ["log", "process_output"])
        return x

    initial_handlers = logger._state.handlers
    logger._state.handlers = [wrap_handler(handler)
                              for handler in initial_handlers]

    log = io.BytesIO()
    # Setup logging for wptrunner that keeps process output and
    # warning+ level logs only
    logger.add_handler(StreamHandler(log, JSONFormatter()))

    wptrunner.run_tests(**kwargs)

    logger._state.handlers = initial_handlers
    logger._state.running_tests = set()
    logger._state.suite_started = False

    log.seek(0)
    results, inconsistent, slow = process_results(log, iterations)
    return results, inconsistent, slow, iterations


def get_steps(logger, repeat_loop, repeat_restart, kwargs_extras):
    steps = []
    for kwargs_extra in kwargs_extras:
        if kwargs_extra:
            flags_string = " with flags %s" % " ".join(
                "%s=%s" % item for item in kwargs_extra.items())
        else:
            flags_string = ""

        if repeat_loop:
            desc = "Running tests in a loop %d times%s" % (repeat_loop,
                                                           flags_string)
            steps.append((desc, functools.partial(run_step, logger, repeat_loop, False, kwargs_extra)))

        if repeat_restart:
            desc = "Running tests in a loop with restarts %s times%s" % (repeat_restart,
                                                                         flags_string)
            steps.append((desc, functools.partial(run_step, logger, repeat_restart, True, kwargs_extra)))

    return steps


def write_summary(logger, step_results, final_result):
    for desc, result in step_results:
        logger.info('::: %s : %s' % (desc, result))
    logger.info(':::')
    if final_result == "PASS":
        log = logger.info
    elif final_result == "TIMEOUT":
        log = logger.warning
    else:
        log = logger.error
    log('::: Test verification %s' % final_result)

    logger.info(':::')

def check_stability(logger, repeat_loop=10, repeat_restart=5, chaos_mode=True, max_time=None,
                    output_results=True, **kwargs):
    kwargs_extras = [{}]
    if chaos_mode and kwargs["product"] == "firefox":
        kwargs_extras.append({"chaos_mode_flags": "0xfb"})

    steps = get_steps(logger, repeat_loop, repeat_restart, kwargs_extras)

    start_time = datetime.now()
    step_results = []

    github_checks_outputter = get_gh_checks_outputter(kwargs["github_checks_text_file"])

    for desc, step_func in steps:
        if max_time and datetime.now() - start_time > max_time:
            logger.info("::: Test verification is taking too long: Giving up!")
            logger.info("::: So far, all checks passed, but not all checks were run.")
            write_summary(logger, step_results, "TIMEOUT")
            return 2

        logger.info(':::')
        logger.info('::: Running test verification step "%s"...' % desc)
        logger.info(':::')
        results, inconsistent, slow, iterations = step_func(**kwargs)
        if output_results:
            write_results(logger.info, results, iterations)

        if inconsistent:
            step_results.append((desc, "FAIL"))
            if github_checks_outputter:
                write_github_checks_summary_inconsistent(github_checks_outputter.output, inconsistent, iterations)
            write_inconsistent(logger.info, inconsistent, iterations)
            write_summary(logger, step_results, "FAIL")
            return 1

        if slow:
            step_results.append((desc, "FAIL"))
            if github_checks_outputter:
                write_github_checks_summary_slow_tests(github_checks_outputter.output, slow)
            write_slow_tests(logger.info, slow)
            write_summary(logger, step_results, "FAIL")
            return 1

        step_results.append((desc, "PASS"))

    write_summary(logger, step_results, "PASS")
