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
from wpt.markdown import markdown_adjust, table


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
            "status": defaultdict(int)
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

    def test_status(self, data):
        subtest = self.find_or_create_subtest(data)
        subtest["status"][data["status"]] += 1
        if data.get("message"):
            subtest["messages"].add(data["message"])

    def test_end(self, data):
        test = self.find_or_create_test(data)
        test["status"][data["status"]] += 1


def is_inconsistent(results_dict, iterations):
    """Return whether or not a single test is inconsistent."""
    if 'SKIP' in results_dict:
        return False
    return len(results_dict) > 1 or sum(results_dict.values()) != iterations


def process_results(log, iterations):
    """Process test log and return overall results and list of inconsistent tests."""
    inconsistent = []
    handler = LogHandler()
    reader.handle_log(reader.read(log), handler)
    results = handler.results
    for test_name, test in results.iteritems():
        if is_inconsistent(test["status"], iterations):
            inconsistent.append((test_name, None, test["status"], []))
        for subtest_name, subtest in test["subtests"].iteritems():
            if is_inconsistent(subtest["status"], iterations):
                inconsistent.append((test_name, subtest_name, subtest["status"], subtest["messages"]))
    return results, inconsistent


def err_string(results_dict, iterations):
    """Create and return string with errors from test run."""
    rv = []
    total_results = sum(results_dict.values())
    for key, value in sorted(results_dict.items()):
        rv.append("%s%s" %
                  (key, ": %s/%s" % (value, iterations) if value != iterations else ""))
    if total_results < iterations:
        rv.append("MISSING: %s/%s" % (iterations - total_results, iterations))
    rv = ", ".join(rv)
    if is_inconsistent(results_dict, iterations):
        rv = "**%s**" % rv
    return rv


def write_inconsistent(log, inconsistent, iterations):
    """Output inconsistent tests to logger.error."""
    log("## Unstable results ##\n")
    strings = [(
        "`%s`" % markdown_adjust(test),
        ("`%s`" % markdown_adjust(subtest)) if subtest else "",
        err_string(results, iterations),
        ("`%s`" % markdown_adjust(";".join(messages))) if len(messages) else "")
        for test, subtest, results, messages in inconsistent]
    table(["Test", "Subtest", "Results", "Messages"], strings, log)


def write_results(log, results, iterations, pr_number=None, use_details=False):
    log("## All results ##\n")
    if use_details:
        log("<details>\n")
        log("<summary>%i %s ran</summary>\n\n" % (len(results),
                                                  "tests" if len(results) > 1
                                                  else "test"))

    for test_name, test in results.iteritems():
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
    import wptrunner
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
    results, inconsistent = process_results(log, iterations)
    return results, inconsistent, iterations


def get_steps(logger, repeat_loop, repeat_restart, kwargs_extras):
    steps = []
    for kwargs_extra in kwargs_extras:
        if kwargs_extra:
            flags_string = " with flags %s" % " ".join(
                "%s=%s" % item for item in kwargs_extra.iteritems())
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
        kwargs_extras.append({"chaos_mode_flags": 3})

    steps = get_steps(logger, repeat_loop, repeat_restart, kwargs_extras)

    start_time = datetime.now()
    step_results = []

    for desc, step_func in steps:
        if max_time and datetime.now() - start_time > max_time:
            logger.info("::: Test verification is taking too long: Giving up!")
            logger.info("::: So far, all checks passed, but not all checks were run.")
            write_summary(logger, step_results, "TIMEOUT")
            return 2

        logger.info(':::')
        logger.info('::: Running test verification step "%s"...' % desc)
        logger.info(':::')
        results, inconsistent, iterations = step_func(**kwargs)
        if output_results:
            write_results(logger.info, results, iterations)

        if inconsistent:
            step_results.append((desc, "FAIL"))
            write_inconsistent(logger.info, inconsistent, iterations)
            write_summary(logger, step_results, "FAIL")
            return 1

        step_results.append((desc, "PASS"))

    write_summary(logger, step_results, "PASS")
