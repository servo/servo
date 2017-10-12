import os
import sys
from collections import OrderedDict, defaultdict

from mozlog import reader
from mozlog.formatters import JSONFormatter, TbplFormatter
from mozlog.handlers import BaseHandler, LogLevelFilter, StreamHandler

from markdown import markdown_adjust, table
from wptrunner import wptrunner


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


def run(venv, logger, **kwargs):
    kwargs["pause_after_test"] = False
    if kwargs["repeat"] == 1:
        kwargs["repeat"] = 10

    handler = LogActionFilter(
        LogLevelFilter(
            StreamHandler(
                sys.stdout,
                TbplFormatter()
            ),
            "WARNING"),
        ["log", "process_output"])

    # There is a public API for this in the next mozlog
    initial_handlers = logger._state.handlers
    logger._state.handlers = []

    with open("raw.log", "wb") as log:
        # Setup logging for wptrunner that keeps process output and
        # warning+ level logs only
        logger.add_handler(handler)
        logger.add_handler(StreamHandler(log, JSONFormatter()))

        wptrunner.run_tests(**kwargs)

    logger._state.handlers = initial_handlers

    with open("raw.log", "rb") as log:
        results, inconsistent = process_results(log, kwargs["repeat"])

    return kwargs["repeat"], results, inconsistent
