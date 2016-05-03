# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import hashlib
import json
import os
import traceback
import urlparse
from abc import ABCMeta, abstractmethod

from ..testrunner import Stop

here = os.path.split(__file__)[0]


def executor_kwargs(test_type, server_config, cache_manager, **kwargs):
    timeout_multiplier = kwargs["timeout_multiplier"]
    if timeout_multiplier is None:
        timeout_multiplier = 1

    executor_kwargs = {"server_config": server_config,
                       "timeout_multiplier": timeout_multiplier,
                       "debug_info": kwargs["debug_info"]}

    if test_type == "reftest":
        executor_kwargs["screenshot_cache"] = cache_manager.dict()

    return executor_kwargs


def strip_server(url):
    """Remove the scheme and netloc from a url, leaving only the path and any query
    or fragment.

    url - the url to strip

    e.g. http://example.org:8000/tests?id=1#2 becomes /tests?id=1#2"""

    url_parts = list(urlparse.urlsplit(url))
    url_parts[0] = ""
    url_parts[1] = ""
    return urlparse.urlunsplit(url_parts)


class TestharnessResultConverter(object):
    harness_codes = {0: "OK",
                     1: "ERROR",
                     2: "TIMEOUT"}

    test_codes = {0: "PASS",
                  1: "FAIL",
                  2: "TIMEOUT",
                  3: "NOTRUN"}

    def __call__(self, test, result):
        """Convert a JSON result into a (TestResult, [SubtestResult]) tuple"""
        result_url, status, message, stack, subtest_results = result
        assert result_url == test.url, ("Got results from %s, expected %s" %
                                      (result_url, test.url))
        harness_result = test.result_cls(self.harness_codes[status], message)
        return (harness_result,
                [test.subtest_result_cls(name, self.test_codes[status], message, stack)
                 for name, status, message, stack in subtest_results])


testharness_result_converter = TestharnessResultConverter()


def reftest_result_converter(self, test, result):
    return (test.result_cls(result["status"], result["message"],
                            extra=result.get("extra")), [])


def pytest_result_converter(self, test, data):
    harness_data, subtest_data = data

    if subtest_data is None:
        subtest_data = []

    harness_result = test.result_cls(*harness_data)
    subtest_results = [test.subtest_result_cls(*item) for item in subtest_data]

    return (harness_result, subtest_results)


class ExecutorException(Exception):
    def __init__(self, status, message):
        self.status = status
        self.message = message


class TestExecutor(object):
    __metaclass__ = ABCMeta

    test_type = None
    convert_result = None

    def __init__(self, browser, server_config, timeout_multiplier=1,
                 debug_info=None):
        """Abstract Base class for object that actually executes the tests in a
        specific browser. Typically there will be a different TestExecutor
        subclass for each test type and method of executing tests.

        :param browser: ExecutorBrowser instance providing properties of the
                        browser that will be tested.
        :param server_config: Dictionary of wptserve server configuration of the
                              form stored in TestEnvironment.external_config
        :param timeout_multiplier: Multiplier relative to base timeout to use
                                   when setting test timeout.
        """
        self.runner = None
        self.browser = browser
        self.server_config = server_config
        self.timeout_multiplier = timeout_multiplier
        self.debug_info = debug_info
        self.last_environment = {"protocol": "http",
                                 "prefs": {}}
        self.protocol = None # This must be set in subclasses

    @property
    def logger(self):
        """StructuredLogger for this executor"""
        if self.runner is not None:
            return self.runner.logger

    def setup(self, runner):
        """Run steps needed before tests can be started e.g. connecting to
        browser instance

        :param runner: TestRunner instance that is going to run the tests"""
        self.runner = runner
        if self.protocol is not None:
            self.protocol.setup(runner)

    def teardown(self):
        """Run cleanup steps after tests have finished"""
        if self.protocol is not None:
            self.protocol.teardown()

    def run_test(self, test):
        """Run a particular test.

        :param test: The test to run"""
        if test.environment != self.last_environment:
            self.on_environment_change(test.environment)

        try:
            result = self.do_test(test)
        except Exception as e:
            result = self.result_from_exception(test, e)

        if result is Stop:
            return result

        # log result of parent test
        if result[0].status == "ERROR":
            self.logger.debug(result[0].message)

        self.last_environment = test.environment

        self.runner.send_message("test_ended", test, result)

    def server_url(self, protocol):
        return "%s://%s:%s" % (protocol,
                               self.server_config["host"],
                               self.server_config["ports"][protocol][0])

    def test_url(self, test):
        return urlparse.urljoin(self.server_url(test.environment["protocol"]), test.url)

    @abstractmethod
    def do_test(self, test):
        """Test-type and protocol specific implementation of running a
        specific test.

        :param test: The test to run."""
        pass

    def on_environment_change(self, new_environment):
        pass

    def result_from_exception(self, test, e):
        if hasattr(e, "status") and e.status in test.result_cls.statuses:
            status = e.status
        else:
            status = "ERROR"
        message = unicode(getattr(e, "message", ""))
        if message:
            message += "\n"
        message += traceback.format_exc(e)
        return test.result_cls(status, message), []


class TestharnessExecutor(TestExecutor):
    convert_result = testharness_result_converter


class RefTestExecutor(TestExecutor):
    convert_result = reftest_result_converter

    def __init__(self, browser, server_config, timeout_multiplier=1, screenshot_cache=None,
                 debug_info=None):
        TestExecutor.__init__(self, browser, server_config,
                              timeout_multiplier=timeout_multiplier,
                              debug_info=debug_info)

        self.screenshot_cache = screenshot_cache


class RefTestImplementation(object):
    def __init__(self, executor):
        self.timeout_multiplier = executor.timeout_multiplier
        self.executor = executor
        # Cache of url:(screenshot hash, screenshot). Typically the
        # screenshot is None, but we set this value if a test fails
        # and the screenshot was taken from the cache so that we may
        # retrieve the screenshot from the cache directly in the future
        self.screenshot_cache = self.executor.screenshot_cache
        self.message = None

    @property
    def logger(self):
        return self.executor.logger

    def get_hash(self, test, viewport_size, dpi):
        timeout = test.timeout * self.timeout_multiplier
        key = (test.url, viewport_size, dpi)

        if key not in self.screenshot_cache:
            success, data = self.executor.screenshot(test, viewport_size, dpi)

            if not success:
                return False, data

            screenshot = data
            hash_value = hashlib.sha1(screenshot).hexdigest()

            self.screenshot_cache[key] = (hash_value, None)

            rv = (hash_value, screenshot)
        else:
            rv = self.screenshot_cache[key]

        self.message.append("%s %s" % (test.url, rv[0]))
        return True, rv

    def is_pass(self, lhs_hash, rhs_hash, relation):
        assert relation in ("==", "!=")
        self.message.append("Testing %s %s %s" % (lhs_hash, relation, rhs_hash))
        return ((relation == "==" and lhs_hash == rhs_hash) or
                (relation == "!=" and lhs_hash != rhs_hash))

    def run_test(self, test):
        viewport_size = test.viewport_size
        dpi = test.dpi
        self.message = []

        # Depth-first search of reference tree, with the goal
        # of reachings a leaf node with only pass results

        stack = list(((test, item[0]), item[1]) for item in reversed(test.references))
        while stack:
            hashes = [None, None]
            screenshots = [None, None]

            nodes, relation = stack.pop()

            for i, node in enumerate(nodes):
                success, data = self.get_hash(node, viewport_size, dpi)
                if success is False:
                    return {"status": data[0], "message": data[1]}

                hashes[i], screenshots[i] = data

            if self.is_pass(hashes[0], hashes[1], relation):
                if nodes[1].references:
                    stack.extend(list(((nodes[1], item[0]), item[1]) for item in reversed(nodes[1].references)))
                else:
                    # We passed
                    return {"status":"PASS", "message": None}

        # We failed, so construct a failure message

        for i, (node, screenshot) in enumerate(zip(nodes, screenshots)):
            if screenshot is None:
                success, screenshot = self.retake_screenshot(node, viewport_size, dpi)
                if success:
                    screenshots[i] = screenshot

        log_data = [{"url": nodes[0].url, "screenshot": screenshots[0]}, relation,
                    {"url": nodes[1].url, "screenshot": screenshots[1]}]

        return {"status": "FAIL",
                "message": "\n".join(self.message),
                "extra": {"reftest_screenshots": log_data}}

    def retake_screenshot(self, node, viewport_size, dpi):
        success, data = self.executor.screenshot(node, viewport_size, dpi)
        if not success:
            return False, data

        key = (node.url, viewport_size, dpi)
        hash_val, _ = self.screenshot_cache[key]
        self.screenshot_cache[key] = hash_val, data
        return True, data


class WdspecExecutor(TestExecutor):
    convert_result = pytest_result_converter


class Protocol(object):
    def __init__(self, executor, browser):
        self.executor = executor
        self.browser = browser

    @property
    def logger(self):
        return self.executor.logger

    def setup(self, runner):
        pass

    def teardown(self):
        pass

    def wait(self):
        pass
