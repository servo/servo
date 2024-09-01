# mypy: allow-untyped-defs

import base64
import hashlib
import io
import json
import os
import threading
import traceback
import socket
import sys
from abc import ABCMeta, abstractmethod
from typing import Any, Callable, ClassVar, Tuple, Type
from urllib.parse import urljoin, urlsplit, urlunsplit

from . import pytestrunner
from .actions import actions
from .asyncactions import async_actions
from .protocol import Protocol, WdspecProtocol


here = os.path.dirname(__file__)


def executor_kwargs(test_type, test_environment, run_info_data, subsuite, **kwargs):
    timeout_multiplier = kwargs["timeout_multiplier"]
    if timeout_multiplier is None:
        timeout_multiplier = 1

    executor_kwargs = {"server_config": test_environment.config,
                       "timeout_multiplier": timeout_multiplier,
                       "debug_info": kwargs["debug_info"],
                       "subsuite": subsuite.name,
                       "target_platform": run_info_data["os"]}

    if test_type in ("reftest", "print-reftest"):
        executor_kwargs["screenshot_cache"] = test_environment.cache_manager.dict()
        executor_kwargs["reftest_screenshot"] = kwargs["reftest_screenshot"]

    if test_type == "wdspec":
        executor_kwargs["binary"] = kwargs["binary"]
        executor_kwargs["binary_args"] = kwargs["binary_args"].copy()
        executor_kwargs["webdriver_binary"] = kwargs["webdriver_binary"]
        executor_kwargs["webdriver_args"] = kwargs["webdriver_args"].copy()

    # By default the executor may try to cleanup windows after a test (to best
    # associate any problems with the test causing them). If the user might
    # want to view the results, however, the executor has to skip that cleanup.
    if kwargs["pause_after_test"] or kwargs["pause_on_unexpected"]:
        executor_kwargs["cleanup_after_test"] = False
    executor_kwargs["debug_test"] = kwargs["debug_test"]
    return executor_kwargs


def strip_server(url):
    """Remove the scheme and netloc from a url, leaving only the path and any query
    or fragment.

    url - the url to strip

    e.g. http://example.org:8000/tests?id=1#2 becomes /tests?id=1#2"""

    url_parts = list(urlsplit(url))
    url_parts[0] = ""
    url_parts[1] = ""
    return urlunsplit(url_parts)


def server_url(server_config, protocol, subdomain=False):
    scheme = "https" if protocol == "h2" else protocol
    host = server_config["browser_host"]
    if subdomain:
        # The only supported subdomain filename flag is "www".
        host = "{subdomain}.{host}".format(subdomain="www", host=host)
    return "{scheme}://{host}:{port}".format(scheme=scheme, host=host,
        port=server_config["ports"][protocol][0])


class TestharnessResultConverter:
    harness_codes = {0: "OK",
                     1: "ERROR",
                     2: "TIMEOUT",
                     3: "PRECONDITION_FAILED"}

    test_codes = {0: "PASS",
                  1: "FAIL",
                  2: "TIMEOUT",
                  3: "NOTRUN",
                  4: "PRECONDITION_FAILED"}

    def __call__(self, test, result, extra=None):
        """Convert a JSON result into a (TestResult, [SubtestResult]) tuple"""
        result_url, status, message, stack, subtest_results = result
        assert result_url == test.url, (f"Got results from {result_url}, expected {test.url}")
        harness_result = test.make_result(self.harness_codes[status], message, extra=extra, stack=stack)
        return (harness_result,
                [test.make_subtest_result(st_name, self.test_codes[st_status], st_message, st_stack)
                 for st_name, st_status, st_message, st_stack in subtest_results])


testharness_result_converter = TestharnessResultConverter()


def hash_screenshots(screenshots):
    """Computes the sha1 checksum of a list of base64-encoded screenshots."""
    return [hashlib.sha1(base64.b64decode(screenshot)).hexdigest()
            for screenshot in screenshots]


def _ensure_hash_in_reftest_screenshots(extra):
    """Make sure reftest_screenshots have hashes.

    Marionette internal reftest runner does not produce hashes.
    """
    log_data = extra.get("reftest_screenshots")
    if not log_data:
        return
    for item in log_data:
        if not isinstance(item, dict):
            # Skip relation strings.
            continue
        if "hash" not in item:
            item["hash"] = hash_screenshots([item["screenshot"]])[0]


def get_pages(ranges_value, total_pages):
    """Get a set of page numbers to include in a print reftest.

    :param ranges_value: Parsed page ranges as a list e.g. [[1,2], [4], [6,None]]
    :param total_pages: Integer total number of pages in the paginated output.
    :retval: Set containing integer page numbers to include in the comparison e.g.
             for the example ranges value and 10 total pages this would be
             {1,2,4,6,7,8,9,10}"""
    if not ranges_value:
        return set(range(1, total_pages + 1))

    rv = set()

    for range_limits in ranges_value:
        if len(range_limits) == 1:
            range_limits = [range_limits[0], range_limits[0]]

        if range_limits[0] is None:
            range_limits[0] = 1
        if range_limits[1] is None:
            range_limits[1] = total_pages

        if range_limits[0] > total_pages:
            continue
        rv |= set(range(range_limits[0], range_limits[1] + 1))
    return rv


def reftest_result_converter(self, test, result):
    extra = result.get("extra", {})
    _ensure_hash_in_reftest_screenshots(extra)
    return (test.make_result(
        result["status"],
        result["message"],
        extra=extra,
        stack=result.get("stack")), [])


def pytest_result_converter(self, test, data):
    harness_data, subtest_data = data

    if subtest_data is None:
        subtest_data = []

    harness_result = test.make_result(*harness_data)
    subtest_results = [test.make_subtest_result(*item) for item in subtest_data]

    return (harness_result, subtest_results)


def crashtest_result_converter(self, test, result):
    return test.make_result(**result), []


class ExecutorException(Exception):
    def __init__(self, status, message):
        self.status = status
        self.message = message


class TimedRunner:
    def __init__(self, logger, func, protocol, url, timeout, extra_timeout):
        self.func = func
        self.logger = logger
        self.result = None
        self.protocol = protocol
        self.url = url
        self.timeout = timeout
        self.extra_timeout = extra_timeout
        self.result_flag = threading.Event()

    def run(self):
        for setup_fn in [self.set_timeout, self.before_run]:
            err = setup_fn()
            if err:
                self.result = (False, err)
                return self.result

        executor = threading.Thread(target=self.run_func)
        executor.start()

        # Add twice the extra timeout since the called function is expected to
        # wait at least self.timeout + self.extra_timeout and this gives some leeway
        timeout = self.timeout + 2 * self.extra_timeout if self.timeout else None
        finished = self.result_flag.wait(timeout)
        if self.result is None:
            if finished:
                # flag is True unless we timeout; this *shouldn't* happen, but
                # it can if self.run_func fails to set self.result due to raising
                self.result = False, ("INTERNAL-ERROR", "%s.run_func didn't set a result" %
                                      self.__class__.__name__)
            else:
                if self.protocol.is_alive():
                    message = "Executor hit external timeout (this may indicate a hang)\n"
                    if executor.ident in sys._current_frames():
                        # get a traceback for the current stack of the executor thread
                        message += "".join(traceback.format_stack(
                            sys._current_frames()[executor.ident]))
                    self.result = False, ("EXTERNAL-TIMEOUT", message)
                else:
                    self.logger.info("Browser not responding, setting status to CRASH")
                    self.result = False, ("CRASH", None)
        elif self.result[1] is None:
            # We didn't get any data back from the test, so check if the
            # browser is still responsive
            if self.protocol.is_alive():
                self.result = False, ("INTERNAL-ERROR", None)
            else:
                self.logger.info("Browser not responding, setting status to CRASH")
                self.result = False, ("CRASH", None)

        return self.result

    def set_timeout(self):
        raise NotImplementedError

    def before_run(self):
        pass

    def run_func(self):
        raise NotImplementedError


class TestExecutor:
    """Abstract Base class for object that actually executes the tests in a
    specific browser. Typically there will be a different TestExecutor
    subclass for each test type and method of executing tests.

    :param browser: ExecutorBrowser instance providing properties of the
                    browser that will be tested.
    :param server_config: Dictionary of wptserve server configuration of the
                          form stored in TestEnvironment.config
    :param timeout_multiplier: Multiplier relative to base timeout to use
                               when setting test timeout.
    """
    __metaclass__ = ABCMeta

    test_type: ClassVar[str]
    # convert_result is a class variable set to a callable converter
    # (e.g. reftest_result_converter) converting from an instance of
    # URLManifestItem (e.g. RefTest) + type-dependent results object +
    # type-dependent extra data, returning a tuple of Result and list of
    # SubtestResult. For now, any callable is accepted. TODO: Make this type
    # stricter when more of the surrounding code is annotated.
    convert_result: ClassVar[Callable[..., Any]]
    supports_testdriver = False
    supports_jsshell = False
    # Extra timeout to use after internal test timeout at which the harness
    # should force a timeout
    extra_timeout = 5  # seconds


    def __init__(self, logger, browser, server_config, timeout_multiplier=1,
                 debug_info=None, subsuite=None, **kwargs):
        self.logger = logger
        self.runner = None
        self.browser = browser
        self.server_config = server_config
        self.timeout_multiplier = timeout_multiplier
        self.debug_info = debug_info
        self.subsuite = subsuite
        self.last_environment = {"protocol": "http",
                                 "prefs": {}}
        self.protocol = None  # This must be set in subclasses

    def setup(self, runner, protocol=None):
        """Run steps needed before tests can be started e.g. connecting to
        browser instance

        :param runner: TestRunner instance that is going to run the tests.
        :param protocol: protocol connection to reuse if not None"""
        self.runner = runner
        if protocol is not None:
            assert isinstance(protocol, self.protocol_cls)
            self.protocol = protocol
        elif self.protocol is not None:
            self.protocol.setup(runner)

    def teardown(self):
        """Run cleanup steps after tests have finished"""
        if self.protocol is not None:
            self.protocol.teardown()

    def reset(self):
        """Re-initialize internal state to facilitate repeated test execution
        as implemented by the `--rerun` command-line argument."""
        pass

    def run_test(self, test):
        """Run a particular test.

        :param test: The test to run"""
        try:
            if test.environment != self.last_environment:
                self.on_environment_change(test.environment)
            result = self.do_test(test)
        except Exception as e:
            exception_string = traceback.format_exc()
            message = f"Exception in TestExecutor.run:\n{exception_string}"
            self.logger.warning(message)
            result = self.result_from_exception(test, e, exception_string)

        # log result of parent test
        if result[0].status == "ERROR":
            self.logger.debug(result[0].message)

        self.last_environment = test.environment

        self.runner.send_message("test_ended", test, result)

    def server_url(self, protocol, subdomain=False):
        return server_url(self.server_config, protocol, subdomain)

    def test_url(self, test):
        return urljoin(self.server_url(test.environment["protocol"],
                                       test.subdomain), test.url)

    @abstractmethod
    def do_test(self, test):
        """Test-type and protocol specific implementation of running a
        specific test.

        :param test: The test to run."""
        pass

    def on_environment_change(self, new_environment):
        pass

    def result_from_exception(self, test, e, exception_string):
        if hasattr(e, "status") and e.status in test.result_cls.statuses:
            status = e.status
        else:
            status = "INTERNAL-ERROR"
        message = str(getattr(e, "message", ""))
        if message:
            message += "\n"
        message += exception_string
        return test.make_result(status, message), []

    def wait(self):
        return self.protocol.base.wait()


class TestharnessExecutor(TestExecutor):
    convert_result = testharness_result_converter


class RefTestExecutor(TestExecutor):
    convert_result = reftest_result_converter
    is_print = False

    def __init__(self, logger, browser, server_config, timeout_multiplier=1, screenshot_cache=None,
                 debug_info=None, reftest_screenshot="unexpected", **kwargs):
        TestExecutor.__init__(self, logger, browser, server_config,
                              timeout_multiplier=timeout_multiplier,
                              debug_info=debug_info)

        self.screenshot_cache = screenshot_cache
        self.reftest_screenshot = reftest_screenshot


class CrashtestExecutor(TestExecutor):
    convert_result = crashtest_result_converter


class PrintRefTestExecutor(TestExecutor):
    convert_result = reftest_result_converter
    is_print = True


class RefTestImplementation:
    def __init__(self, executor):
        self.timeout_multiplier = executor.timeout_multiplier
        self.executor = executor
        self.subsuite = executor.subsuite
        # Cache of url:(screenshot hash, screenshot). Typically the
        # screenshot is None, but we set this value if a test fails
        # and the screenshot was taken from the cache so that we may
        # retrieve the screenshot from the cache directly in the future
        self.screenshot_cache = self.executor.screenshot_cache
        self.message = None
        self.reftest_screenshot = executor.reftest_screenshot

    def setup(self):
        pass

    def teardown(self):
        pass

    @property
    def logger(self):
        return self.executor.logger

    def get_hash(self, test, viewport_size, dpi, page_ranges):
        key = (self.subsuite, test.url, viewport_size, dpi)

        if key not in self.screenshot_cache:
            success, data = self.get_screenshot_list(test, viewport_size, dpi, page_ranges)

            if not success:
                return False, data

            screenshots = data
            hash_values = hash_screenshots(data)
            self.screenshot_cache[key] = (hash_values, screenshots)

            rv = (hash_values, screenshots)
        else:
            rv = self.screenshot_cache[key]

        self.message.append(f"{test.url} {rv[0]}")
        return True, rv

    def reset(self):
        self.screenshot_cache.clear()

    def check_pass(self, hashes, screenshots, urls, relation, fuzzy):
        """Check if a test passes, and return a tuple of (pass, page_idx),
        where page_idx is the zero-based index of the first page on which a
        difference occurs if any, or None if there are no differences"""

        assert relation in ("==", "!=")
        lhs_hashes, rhs_hashes = hashes
        lhs_screenshots, rhs_screenshots = screenshots

        if len(lhs_hashes) != len(rhs_hashes):
            self.logger.info("Got different number of pages")
            return relation == "!=", -1

        assert len(lhs_screenshots) == len(lhs_hashes) == len(rhs_screenshots) == len(rhs_hashes)

        for (page_idx, (lhs_hash,
                        rhs_hash,
                        lhs_screenshot,
                        rhs_screenshot)) in enumerate(zip(lhs_hashes,
                                                          rhs_hashes,
                                                          lhs_screenshots,
                                                          rhs_screenshots)):
            comparison_screenshots = (lhs_screenshot, rhs_screenshot)
            if not fuzzy or fuzzy == ((0, 0), (0, 0)):
                equal = lhs_hash == rhs_hash
                # sometimes images can have different hashes, but pixels can be identical.
                if not equal:
                    self.logger.info("Image hashes didn't match%s, checking pixel differences" %
                                     ("" if len(hashes) == 1 else " on page %i" % (page_idx + 1)))
                    max_per_channel, pixels_different = self.get_differences(comparison_screenshots,
                                                                             urls)
                    equal = pixels_different == 0 and max_per_channel == 0
            else:
                max_per_channel, pixels_different = self.get_differences(comparison_screenshots,
                                                                         urls,
                                                                         page_idx if len(hashes) > 1 else None)
                allowed_per_channel, allowed_different = fuzzy
                self.logger.info("Allowed %s pixels different, maximum difference per channel %s" %
                                 ("-".join(str(item) for item in allowed_different),
                                  "-".join(str(item) for item in allowed_per_channel)))
                equal = ((pixels_different == 0 and allowed_different[0] == 0) or
                         (max_per_channel == 0 and allowed_per_channel[0] == 0) or
                         (allowed_per_channel[0] <= max_per_channel <= allowed_per_channel[1] and
                          allowed_different[0] <= pixels_different <= allowed_different[1]))
            if not equal:
                return (False if relation == "==" else True, page_idx)
        # All screenshots were equal within the fuzziness
        return (True if relation == "==" else False, -1)

    def get_differences(self, screenshots, urls, page_idx=None):
        from PIL import Image, ImageChops, ImageStat

        lhs = Image.open(io.BytesIO(base64.b64decode(screenshots[0]))).convert("RGB")
        rhs = Image.open(io.BytesIO(base64.b64decode(screenshots[1]))).convert("RGB")
        self.check_if_solid_color(lhs, urls[0])
        self.check_if_solid_color(rhs, urls[1])
        diff = ImageChops.difference(lhs, rhs)
        minimal_diff = diff.crop(diff.getbbox())
        mask = minimal_diff.convert("L", dither=None)
        stat = ImageStat.Stat(minimal_diff, mask)
        per_channel = max(item[1] for item in stat.extrema)
        count = stat.count[0]
        self.logger.info("Found %s pixels different, maximum difference per channel %s%s" %
                         (count,
                          per_channel,
                          "" if page_idx is None else " on page %i" % (page_idx + 1)))
        return per_channel, count

    def check_if_solid_color(self, image, url):
        extrema = image.getextrema()
        if all(min == max for min, max in extrema):
            color = ''.join('%02X' % value for value, _ in extrema)
            self.message.append(f"Screenshot is solid color 0x{color} for {url}\n")

    def run_test(self, test):
        viewport_size = test.viewport_size
        dpi = test.dpi
        page_ranges = test.page_ranges
        self.message = []


        # Depth-first search of reference tree, with the goal
        # of reachings a leaf node with only pass results

        stack = list(((test, item[0]), item[1]) for item in reversed(test.references))

        while stack:
            hashes = [None, None]
            screenshots = [None, None]
            urls = [None, None]

            nodes, relation = stack.pop()
            fuzzy = self.get_fuzzy(test, nodes, relation)

            for i, node in enumerate(nodes):
                success, data = self.get_hash(node, viewport_size, dpi, page_ranges)
                if success is False:
                    return {"status": data[0], "message": data[1]}

                hashes[i], screenshots[i] = data
                urls[i] = node.url

            is_pass, page_idx = self.check_pass(hashes, screenshots, urls, relation, fuzzy)
            log_data = [
                {"url": urls[0], "screenshot": screenshots[0][page_idx],
                 "hash": hashes[0][page_idx]},
                relation,
                {"url": urls[1], "screenshot": screenshots[1][page_idx],
                 "hash": hashes[1][page_idx]}
            ]

            if is_pass:
                fuzzy = self.get_fuzzy(test, nodes, relation)
                if nodes[1].references:
                    stack.extend(list(((nodes[1], item[0]), item[1])
                                      for item in reversed(nodes[1].references)))
                else:
                    test_result = {"status": "PASS", "message": None}
                    if (self.reftest_screenshot == "always" or
                        self.reftest_screenshot == "unexpected" and
                        test.expected() != "PASS"):
                        test_result["extra"] = {"reftest_screenshots": log_data}
                    # We passed
                    return test_result

        # We failed, so construct a failure message

        for i, (node, screenshot) in enumerate(zip(nodes, screenshots)):
            if screenshot is None:
                success, screenshot = self.retake_screenshot(node, viewport_size, dpi, page_ranges)
                if success:
                    screenshots[i] = screenshot

        test_result = {"status": "FAIL",
                       "message": "\n".join(self.message)}
        if (self.reftest_screenshot in ("always", "fail") or
            self.reftest_screenshot == "unexpected" and
            test.expected() != "FAIL"):
            test_result["extra"] = {"reftest_screenshots": log_data}
        return test_result

    def get_fuzzy(self, root_test, test_nodes, relation):
        full_key = tuple([item.url for item in test_nodes] + [relation])
        ref_only_key = test_nodes[1].url

        fuzzy_override = root_test.fuzzy_override
        fuzzy = test_nodes[0].fuzzy

        sources = [fuzzy_override, fuzzy]
        keys = [full_key, ref_only_key, None]
        value = None
        for source in sources:
            for key in keys:
                if key in source:
                    value = source[key]
                    break
            if value:
                break
        return value

    def retake_screenshot(self, node, viewport_size, dpi, page_ranges):
        success, data = self.get_screenshot_list(node,
                                                 viewport_size,
                                                 dpi,
                                                 page_ranges)
        if not success:
            return False, data

        key = (node.url, viewport_size, dpi)
        hash_val, _ = self.screenshot_cache[key]
        self.screenshot_cache[key] = hash_val, data
        return True, data

    def get_screenshot_list(self, node, viewport_size, dpi, page_ranges):
        success, data = self.executor.screenshot(node, viewport_size, dpi, page_ranges)
        if success and not isinstance(data, list):
            return success, [data]
        return success, data


class WdspecExecutor(TestExecutor):
    convert_result = pytest_result_converter
    protocol_cls: ClassVar[Type[Protocol]] = WdspecProtocol

    def __init__(self, logger, browser, server_config, webdriver_binary,
                 webdriver_args, target_platform, timeout_multiplier=1, capabilities=None,
                 debug_info=None, binary=None, binary_args=None, **kwargs):
        super().__init__(logger, browser, server_config,
                         timeout_multiplier=timeout_multiplier,
                         debug_info=debug_info)
        self.webdriver_binary = webdriver_binary
        self.webdriver_args = webdriver_args
        self.timeout_multiplier = timeout_multiplier
        self.capabilities = capabilities
        self.binary = binary
        self.binary_args = binary_args

        # Map OS to WebDriver specific platform names
        os_map = {"win": "windows"}
        self.target_platform = os_map.get(target_platform, target_platform)

    def setup(self, runner, protocol=None):
        assert protocol is None, "Switch executor not allowed for wdspec tests."
        self.protocol = self.protocol_cls(self, self.browser)
        super().setup(runner)

    def is_alive(self):
        return self.protocol.is_alive()

    def on_environment_change(self, new_environment):
        pass

    def do_test(self, test):
        timeout = test.timeout * self.timeout_multiplier + self.extra_timeout

        success, data = WdspecRun(self.do_wdspec,
                                  test.abs_path,
                                  timeout).run()

        if success:
            return self.convert_result(test, data)

        return (test.make_result(*data), [])

    def do_wdspec(self, path, timeout):
        session_config = {"host": self.browser.host,
                          "port": self.browser.port,
                          "capabilities": self.capabilities,
                          "target_platform": self.target_platform,
                          "timeout_multiplier": self.timeout_multiplier,
                          "browser": {
                              "binary": self.binary,
                              "args": self.binary_args,
                              "env": self.browser.env,
                          },
                          "webdriver": {
                              "binary": self.webdriver_binary,
                              "args": self.webdriver_args
                          }}

        return pytestrunner.run(path,
                                self.server_config,
                                session_config,
                                timeout=timeout)


class WdspecRun:
    def __init__(self, func, path, timeout):
        self.func = func
        self.result = (None, None)
        self.path = path
        self.timeout = timeout
        self.result_flag = threading.Event()

    def run(self):
        """Runs function in a thread and interrupts it if it exceeds the
        given timeout.  Returns (True, (Result, [SubtestResult ...])) in
        case of success, or (False, (status, extra information)) in the
        event of failure.
        """

        executor = threading.Thread(target=self._run)
        executor.start()

        self.result_flag.wait(self.timeout)
        if self.result[1] is None:
            self.result = False, ("EXTERNAL-TIMEOUT", None)

        return self.result

    def _run(self):
        try:
            self.result = True, self.func(self.path, self.timeout)
        except (socket.timeout, OSError):
            self.result = False, ("CRASH", None)
        except Exception as e:
            message = getattr(e, "message", "")
            if message:
                message += "\n"
            message += traceback.format_exc()
            self.result = False, ("INTERNAL-ERROR", message)
        finally:
            self.result_flag.set()


class CallbackHandler:
    """Handle callbacks from testdriver-using tests.

    The default implementation here makes sense for things that are roughly like
    WebDriver. Things that are more different to WebDriver may need to create a
    fully custom implementation."""

    unimplemented_exc: ClassVar[Tuple[Type[Exception], ...]] = (NotImplementedError,)
    expected_exc: ClassVar[Tuple[Type[Exception], ...]] = ()

    def __init__(self, logger, protocol, test_window):
        self.protocol = protocol
        self.test_window = test_window
        self.logger = logger
        self.callbacks = {
            "action": self.process_action,
            "complete": self.process_complete
        }

        self.actions = {cls.name: cls(self.logger, self.protocol) for cls in actions}

    def __call__(self, result):
        url, command, payload = result
        self.logger.debug("Got async callback: %s" % result[1])
        try:
            callback = self.callbacks[command]
        except KeyError as e:
            raise ValueError("Unknown callback type %r" % result[1]) from e
        return callback(url, payload)

    def process_complete(self, url, payload):
        rv = [strip_server(url)] + payload
        return True, rv

    def process_action(self, url, payload):
        action = payload["action"]
        cmd_id = payload["id"]
        self.logger.debug(f"Got action: {action}")
        try:
            action_handler = self.actions[action]
        except KeyError as e:
            raise ValueError(f"Unknown action {action}") from e
        try:
            with ActionContext(self.logger, self.protocol, payload.get("context")):
                try:
                    result = action_handler(payload)
                except AttributeError as e:
                    # If we fail to get an attribute from the protocol presumably that's a
                    # ProtocolPart we don't implement
                    # AttributeError got an obj property in Python 3.10, for older versions we
                    # fall back to looking at the error message.
                    if ((hasattr(e, "obj") and getattr(e, "obj") == self.protocol) or
                        f"'{self.protocol.__class__.__name__}' object has no attribute" in str(e)):
                        raise NotImplementedError from e
                    raise
        except self.unimplemented_exc:
            self.logger.warning("Action %s not implemented" % action)
            self._send_message(cmd_id, "complete", "error", f"Action {action} not implemented")
        except self.expected_exc:
            self.logger.debug(f"Action {action} failed with an expected exception")
            self._send_message(cmd_id, "complete", "error", f"Action {action} failed")
        except Exception:
            self.logger.warning(f"Action {action} failed")
            self._send_message(cmd_id, "complete", "error")
            raise
        else:
            self.logger.debug(f"Action {action} completed with result {result}")
            return_message = {"result": result}
            self._send_message(cmd_id, "complete", "success", json.dumps(return_message))

        return False, None

    def _send_message(self, cmd_id, message_type, status, message=None):
        self.protocol.testdriver.send_message(cmd_id, message_type, status, message=message)


class AsyncCallbackHandler(CallbackHandler):
    """
    Handle synchronous and asynchronous actions. Extends `CallbackHandler` with support of async actions.
    """

    def __init__(self, logger, protocol, test_window, loop):
        super().__init__(logger, protocol, test_window)
        self.loop = loop
        self.async_actions = {cls.name: cls(self.logger, self.protocol) for cls in async_actions}

    def process_action(self, url, payload):
        action = payload["action"]
        if action in self.async_actions:
            # Schedule async action to be processed in the event loop and return immediately.
            self.logger.debug(f"Scheduling async action processing: {action}, {payload}")
            self.loop.create_task(self._process_async_action(action, payload))
            return False, None
        else:
            # Fallback to the default action processing, which will fail if the action is not implemented.
            self.logger.debug(f"Processing synchronous action: {action}, {payload}")
            return super().process_action(url, payload)

    async def _process_async_action(self, action, payload):
        """
        Process async action and send the result back to the test driver.

        This method is analogous to `process_action` but is intended to be used with async actions in a task, so it does
        not raise unexpected exceptions. However, the unexpected exceptions are logged and the error message is sent
        back to the test driver.
        """
        async_action_handler = self.async_actions[action]
        cmd_id = payload["id"]
        try:
            result = await async_action_handler(payload)
        except AttributeError as e:
            # If we fail to get an attribute from the protocol presumably that's a
            # ProtocolPart we don't implement
            # AttributeError got an obj property in Python 3.10, for older versions we
            # fall back to looking at the error message.
            if ((hasattr(e, "obj") and getattr(e, "obj") == self.protocol) or
                    f"'{self.protocol.__class__.__name__}' object has no attribute" in str(e)):
                raise NotImplementedError from e
        except self.unimplemented_exc:
            self.logger.warning("Action %s not implemented" % action)
            self._send_message(cmd_id, "complete", "error", f"Action {action} not implemented")
        except self.expected_exc as e:
            self.logger.debug(f"Action {action} failed with an expected exception: {e}")
            self._send_message(cmd_id, "complete", "error", f"Action {action} failed: {e}")
        except Exception as e:
            self.logger.warning(f"Action {action} failed with an unexpected exception: {e}")
            self._send_message(cmd_id, "complete", "error", f"Unexpected exception: {e}")
        else:
            self.logger.debug(f"Action {action} completed with result {result}")
            return_message = {"result": result}
            self._send_message(cmd_id, "complete", "success", json.dumps(return_message))


class ActionContext:
    def __init__(self, logger, protocol, context):
        self.logger = logger
        self.protocol = protocol
        self.context = context
        self.initial_window = None

    def __enter__(self):
        if self.context is None:
            return

        self.initial_window = self.protocol.base.current_window
        self.logger.debug("Switching to window %s" % self.context)
        self.protocol.testdriver.switch_to_window(self.context, self.initial_window)

    def __exit__(self, *args):
        if self.context is None:
            return

        self.logger.debug("Switching back to initial window")
        self.protocol.base.set_window(self.initial_window)
        self.initial_window = None
