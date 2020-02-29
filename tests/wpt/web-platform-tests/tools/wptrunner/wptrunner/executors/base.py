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
from six import text_type
from six.moves.http_client import HTTPConnection
from six.moves.urllib.parse import urljoin, urlsplit, urlunsplit

from ..testrunner import Stop
from .protocol import Protocol, BaseProtocolPart

here = os.path.split(__file__)[0]


def executor_kwargs(test_type, server_config, cache_manager, run_info_data,
                    **kwargs):
    timeout_multiplier = kwargs["timeout_multiplier"]
    if timeout_multiplier is None:
        timeout_multiplier = 1

    executor_kwargs = {"server_config": server_config,
                       "timeout_multiplier": timeout_multiplier,
                       "debug_info": kwargs["debug_info"]}

    if test_type == "reftest":
        executor_kwargs["screenshot_cache"] = cache_manager.dict()

    if test_type == "wdspec":
        executor_kwargs["binary"] = kwargs.get("binary")
        executor_kwargs["webdriver_binary"] = kwargs.get("webdriver_binary")
        executor_kwargs["webdriver_args"] = kwargs.get("webdriver_args")

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


class TestharnessResultConverter(object):
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
        assert result_url == test.url, ("Got results from %s, expected %s" %
                                        (result_url, test.url))
        harness_result = test.result_cls(self.harness_codes[status], message, extra=extra, stack=stack)
        return (harness_result,
                [test.subtest_result_cls(st_name, self.test_codes[st_status], st_message, st_stack)
                 for st_name, st_status, st_message, st_stack in subtest_results])


testharness_result_converter = TestharnessResultConverter()


def hash_screenshot(data):
    """Computes the sha1 checksum of a base64-encoded screenshot."""
    return hashlib.sha1(base64.b64decode(data)).hexdigest()


def _ensure_hash_in_reftest_screenshots(extra):
    """Make sure reftest_screenshots have hashes.

    Marionette internal reftest runner does not produce hashes.
    """
    log_data = extra.get("reftest_screenshots")
    if not log_data:
        return
    for item in log_data:
        if type(item) != dict:
            # Skip relation strings.
            continue
        if "hash" not in item:
            item["hash"] = hash_screenshot(item["screenshot"])


def reftest_result_converter(self, test, result):
    extra = result.get("extra", {})
    _ensure_hash_in_reftest_screenshots(extra)
    return (test.result_cls(
        result["status"],
        result["message"],
        extra=extra,
        stack=result.get("stack")), [])


def pytest_result_converter(self, test, data):
    harness_data, subtest_data = data

    if subtest_data is None:
        subtest_data = []

    harness_result = test.result_cls(*harness_data)
    subtest_results = [test.subtest_result_cls(*item) for item in subtest_data]

    return (harness_result, subtest_results)


def crashtest_result_converter(self, test, result):
    return test.result_cls(**result), []

class ExecutorException(Exception):
    def __init__(self, status, message):
        self.status = status
        self.message = message


class TimedRunner(object):
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
        if self.set_timeout() is Stop:
            return Stop

        if self.before_run() is Stop:
            return Stop

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
                    # get a traceback for the current stack of the executor thread
                    message += "".join(traceback.format_stack(sys._current_frames()[executor.ident]))
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


class TestExecutor(object):
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

    test_type = None
    convert_result = None
    supports_testdriver = False
    supports_jsshell = False
    # Extra timeout to use after internal test timeout at which the harness
    # should force a timeout
    extra_timeout = 5  # seconds


    def __init__(self, browser, server_config, timeout_multiplier=1,
                 debug_info=None, **kwargs):
        self.runner = None
        self.browser = browser
        self.server_config = server_config
        self.timeout_multiplier = timeout_multiplier
        self.debug_info = debug_info
        self.last_environment = {"protocol": "http",
                                 "prefs": {}}
        self.protocol = None  # This must be set in subclasses

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

    def reset(self):
        """Re-initialize internal state to facilitate repeated test execution
        as implemented by the `--rerun` command-line argument."""
        pass

    def run_test(self, test):
        """Run a particular test.

        :param test: The test to run"""
        if test.environment != self.last_environment:
            self.on_environment_change(test.environment)
        try:
            result = self.do_test(test)
        except Exception as e:
            exception_string = traceback.format_exc()
            self.logger.warning(exception_string)
            result = self.result_from_exception(test, e, exception_string)

        if result is Stop:
            return result

        # log result of parent test
        if result[0].status == "ERROR":
            self.logger.debug(result[0].message)

        self.last_environment = test.environment

        self.runner.send_message("test_ended", test, result)

    def server_url(self, protocol):
        scheme = "https" if protocol == "h2" else protocol
        return "%s://%s:%s" % (scheme,
                               self.server_config["browser_host"],
                               self.server_config["ports"][protocol][0])

    def test_url(self, test):
        return urljoin(self.server_url(test.environment["protocol"]), test.url)

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
        message = text_type(getattr(e, "message", ""))
        if message:
            message += "\n"
        message += exception_string
        return test.result_cls(status, message), []

    def wait(self):
        self.protocol.base.wait()


class TestharnessExecutor(TestExecutor):
    convert_result = testharness_result_converter


class RefTestExecutor(TestExecutor):
    convert_result = reftest_result_converter

    def __init__(self, browser, server_config, timeout_multiplier=1, screenshot_cache=None,
                 debug_info=None, **kwargs):
        TestExecutor.__init__(self, browser, server_config,
                              timeout_multiplier=timeout_multiplier,
                              debug_info=debug_info)

        self.screenshot_cache = screenshot_cache


class CrashtestExecutor(TestExecutor):
    convert_result = crashtest_result_converter


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

    def setup(self):
        pass

    def teardown(self):
        pass

    @property
    def logger(self):
        return self.executor.logger

    def get_hash(self, test, viewport_size, dpi):
        key = (test.url, viewport_size, dpi)

        if key not in self.screenshot_cache:
            success, data = self.executor.screenshot(test, viewport_size, dpi)

            if not success:
                return False, data

            screenshot = data
            hash_value = hash_screenshot(data)
            self.screenshot_cache[key] = (hash_value, screenshot)

            rv = (hash_value, screenshot)
        else:
            rv = self.screenshot_cache[key]

        self.message.append("%s %s" % (test.url, rv[0]))
        return True, rv

    def reset(self):
        self.screenshot_cache.clear()

    def is_pass(self, hashes, screenshots, urls, relation, fuzzy):
        assert relation in ("==", "!=")
        if not fuzzy or fuzzy == ((0,0), (0,0)):
            equal = hashes[0] == hashes[1]
            # sometimes images can have different hashes, but pixels can be identical.
            if not equal:
                self.logger.info("Image hashes didn't match, checking pixel differences")
                max_per_channel, pixels_different = self.get_differences(screenshots, urls)
                equal = pixels_different == 0 and max_per_channel == 0
        else:
            max_per_channel, pixels_different = self.get_differences(screenshots, urls)
            allowed_per_channel, allowed_different = fuzzy
            self.logger.info("Allowed %s pixels different, maximum difference per channel %s" %
                             ("-".join(str(item) for item in allowed_different),
                              "-".join(str(item) for item in allowed_per_channel)))
            equal = ((pixels_different == 0 and allowed_different[0] == 0) or
                     (max_per_channel == 0 and allowed_per_channel[0] == 0) or
                     (allowed_per_channel[0] <= max_per_channel <= allowed_per_channel[1] and
                      allowed_different[0] <= pixels_different <= allowed_different[1]))
        return equal if relation == "==" else not equal

    def get_differences(self, screenshots, urls):
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
        self.logger.info("Found %s pixels different, maximum difference per channel %s" %
                         (count, per_channel))
        return per_channel, count

    def check_if_solid_color(self, image, url):
        extrema = image.getextrema()
        if all(min == max for min, max in extrema):
            color = ''.join('%02X' % value for value, _ in extrema)
            self.message.append("Screenshot is solid color 0x%s for %s\n" % (color, url))

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
            urls = [None, None]

            nodes, relation = stack.pop()
            fuzzy = self.get_fuzzy(test, nodes, relation)

            for i, node in enumerate(nodes):
                success, data = self.get_hash(node, viewport_size, dpi)
                if success is False:
                    return {"status": data[0], "message": data[1]}

                hashes[i], screenshots[i] = data
                urls[i] = node.url

            if self.is_pass(hashes, screenshots, urls, relation, fuzzy):
                fuzzy = self.get_fuzzy(test, nodes, relation)
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

        log_data = [
            {"url": nodes[0].url, "screenshot": screenshots[0], "hash": hashes[0]},
            relation,
            {"url": nodes[1].url, "screenshot": screenshots[1], "hash": hashes[1]},
        ]

        return {"status": "FAIL",
                "message": "\n".join(self.message),
                "extra": {"reftest_screenshots": log_data}}

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
    protocol_cls = None

    def __init__(self, browser, server_config, webdriver_binary,
                 webdriver_args, timeout_multiplier=1, capabilities=None,
                 debug_info=None, **kwargs):
        self.do_delayed_imports()
        TestExecutor.__init__(self, browser, server_config,
                              timeout_multiplier=timeout_multiplier,
                              debug_info=debug_info)
        self.webdriver_binary = webdriver_binary
        self.webdriver_args = webdriver_args
        self.timeout_multiplier = timeout_multiplier
        self.capabilities = capabilities
        self.protocol = self.protocol_cls(self, browser)

    def is_alive(self):
        return self.protocol.is_alive()

    def on_environment_change(self, new_environment):
        pass

    def do_test(self, test):
        timeout = test.timeout * self.timeout_multiplier + self.extra_timeout

        success, data = WdspecRun(self.do_wdspec,
                                  self.protocol.session_config,
                                  test.abs_path,
                                  timeout).run()

        if success:
            return self.convert_result(test, data)

        return (test.result_cls(*data), [])

    def do_wdspec(self, session_config, path, timeout):
        return pytestrunner.run(path,
                                self.server_config,
                                session_config,
                                timeout=timeout)

    def do_delayed_imports(self):
        global pytestrunner
        from . import pytestrunner


class WdspecRun(object):
    def __init__(self, func, session, path, timeout):
        self.func = func
        self.result = (None, None)
        self.session = session
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
            self.result = True, self.func(self.session, self.path, self.timeout)
        except (socket.timeout, IOError):
            self.result = False, ("CRASH", None)
        except Exception as e:
            message = getattr(e, "message")
            if message:
                message += "\n"
            message += traceback.format_exc()
            self.result = False, ("INTERNAL-ERROR", message)
        finally:
            self.result_flag.set()


class ConnectionlessBaseProtocolPart(BaseProtocolPart):
    def load(self, url):
        pass

    def execute_script(self, script, asynchronous=False):
        pass

    def set_timeout(self, timeout):
        pass

    def wait(self):
        pass

    def set_window(self, handle):
        pass


class ConnectionlessProtocol(Protocol):
    implements = [ConnectionlessBaseProtocolPart]

    def connect(self):
        pass

    def after_connect(self):
        pass


class WebDriverProtocol(Protocol):
    server_cls = None

    implements = [ConnectionlessBaseProtocolPart]

    def __init__(self, executor, browser):
        Protocol.__init__(self, executor, browser)
        self.webdriver_binary = executor.webdriver_binary
        self.webdriver_args = executor.webdriver_args
        self.capabilities = self.executor.capabilities
        self.session_config = None
        self.server = None

    def connect(self):
        """Connect to browser via the HTTP server."""
        self.server = self.server_cls(
            self.logger,
            binary=self.webdriver_binary,
            args=self.webdriver_args)
        self.server.start(block=False)
        self.logger.info(
            "WebDriver HTTP server listening at %s" % self.server.url)
        self.session_config = {"host": self.server.host,
                               "port": self.server.port,
                               "capabilities": self.capabilities}

    def after_connect(self):
        pass

    def teardown(self):
        if self.server is not None and self.server.is_alive():
            self.server.stop()

    def is_alive(self):
        """Test that the connection is still alive.

        Because the remote communication happens over HTTP we need to
        make an explicit request to the remote.  It is allowed for
        WebDriver spec tests to not have a WebDriver session, since this
        may be what is tested.

        An HTTP request to an invalid path that results in a 404 is
        proof enough to us that the server is alive and kicking.
        """
        conn = HTTPConnection(self.server.host, self.server.port)
        conn.request("HEAD", self.server.base_path + "invalid")
        res = conn.getresponse()
        return res.status == 404


class CallbackHandler(object):
    """Handle callbacks from testdriver-using tests.

    The default implementation here makes sense for things that are roughly like
    WebDriver. Things that are more different to WebDriver may need to create a
    fully custom implementation."""

    unimplemented_exc = (NotImplementedError,)

    def __init__(self, logger, protocol, test_window):
        self.protocol = protocol
        self.test_window = test_window
        self.logger = logger
        self.callbacks = {
            "action": self.process_action,
            "complete": self.process_complete
        }

        self.actions = {
            "click": ClickAction(self.logger, self.protocol),
            "send_keys": SendKeysAction(self.logger, self.protocol),
            "action_sequence": ActionSequenceAction(self.logger, self.protocol),
            "generate_test_report": GenerateTestReportAction(self.logger, self.protocol),
            "set_permission": SetPermissionAction(self.logger, self.protocol),
            "add_virtual_authenticator": AddVirtualAuthenticatorAction(self.logger, self.protocol),
            "remove_virtual_authenticator": RemoveVirtualAuthenticatorAction(self.logger, self.protocol),
            "add_credential": AddCredentialAction(self.logger, self.protocol),
            "get_credentials": GetCredentialsAction(self.logger, self.protocol),
            "remove_credential": RemoveCredentialAction(self.logger, self.protocol),
            "remove_all_credentials": RemoveAllCredentialsAction(self.logger, self.protocol),
            "set_user_verified": SetUserVerifiedAction(self.logger, self.protocol),
        }

    def __call__(self, result):
        url, command, payload = result
        self.logger.debug("Got async callback: %s" % result[1])
        try:
            callback = self.callbacks[command]
        except KeyError:
            raise ValueError("Unknown callback type %r" % result[1])
        return callback(url, payload)

    def process_complete(self, url, payload):
        rv = [strip_server(url)] + payload
        return True, rv

    def process_action(self, url, payload):
        action = payload["action"]
        self.logger.debug("Got action: %s" % action)
        try:
            action_handler = self.actions[action]
        except KeyError:
            raise ValueError("Unknown action %s" % action)
        try:
            result = action_handler(payload)
        except self.unimplemented_exc:
            self.logger.warning("Action %s not implemented" % action)
            self._send_message("complete", "error", "Action %s not implemented" % action)
        except Exception:
            self.logger.warning("Action %s failed" % action)
            self.logger.warning(traceback.format_exc())
            self._send_message("complete", "error")
            raise
        else:
            self.logger.debug("Action %s completed with result %s" % (action, result))
            return_message = {"result": result}
            self._send_message("complete", "success", json.dumps(return_message))

        return False, None

    def _send_message(self, message_type, status, message=None):
        self.protocol.testdriver.send_message(message_type, status, message=message)


class ClickAction(object):
    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        selector = payload["selector"]
        element = self.protocol.select.element_by_selector(selector)
        self.logger.debug("Clicking element: %s" % selector)
        self.protocol.click.element(element)


class SendKeysAction(object):
    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        selector = payload["selector"]
        keys = payload["keys"]
        element = self.protocol.select.element_by_selector(selector)
        self.logger.debug("Sending keys to element: %s" % selector)
        self.protocol.send_keys.send_keys(element, keys)


class ActionSequenceAction(object):
    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        # TODO: some sort of shallow error checking
        actions = payload["actions"]
        for actionSequence in actions:
            if actionSequence["type"] == "pointer":
                for action in actionSequence["actions"]:
                    if (action["type"] == "pointerMove" and
                        isinstance(action["origin"], dict)):
                        action["origin"] = self.get_element(action["origin"]["selector"], action["frame"]["frame"])
        self.protocol.action_sequence.send_actions({"actions": actions})

    def get_element(self, element_selector, frame):
        element = self.protocol.select.element_by_selector(element_selector, frame)
        return element

class GenerateTestReportAction(object):
    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        message = payload["message"]
        self.logger.debug("Generating test report: %s" % message)
        self.protocol.generate_test_report.generate_test_report(message)

class SetPermissionAction(object):
    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        permission_params = payload["permission_params"]
        descriptor = permission_params["descriptor"]
        name = descriptor["name"]
        state = permission_params["state"]
        one_realm = permission_params.get("oneRealm", False)
        self.logger.debug("Setting permission %s to %s, oneRealm=%s" % (name, state, one_realm))
        self.protocol.set_permission.set_permission(descriptor, state, one_realm)

class AddVirtualAuthenticatorAction(object):
    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        self.logger.debug("Adding virtual authenticator")
        config = payload["config"]
        authenticator_id = self.protocol.virtual_authenticator.add_virtual_authenticator(config)
        self.logger.debug("Authenticator created with ID %s" % authenticator_id)
        return authenticator_id

class RemoveVirtualAuthenticatorAction(object):
    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        self.logger.debug("Removing virtual authenticator %s" % authenticator_id)
        return self.protocol.virtual_authenticator.remove_virtual_authenticator(authenticator_id)


class AddCredentialAction(object):
    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        credential = payload["credential"]
        self.logger.debug("Adding credential to virtual authenticator %s " % authenticator_id)
        return self.protocol.virtual_authenticator.add_credential(authenticator_id, credential)

class GetCredentialsAction(object):
    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        self.logger.debug("Getting credentials from virtual authenticator %s " % authenticator_id)
        return self.protocol.virtual_authenticator.get_credentials(authenticator_id)

class RemoveCredentialAction(object):
    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        credential_id = payload["credential_id"]
        self.logger.debug("Removing credential %s from authenticator %s" % (credential_id, authenticator_id))
        return self.protocol.virtual_authenticator.remove_credential(authenticator_id, credential_id)

class RemoveAllCredentialsAction(object):
    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        self.logger.debug("Removing all credentials from authenticator %s" % authenticator_id)
        return self.protocol.virtual_authenticator.remove_all_credentials(authenticator_id)

class SetUserVerifiedAction(object):
    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        authenticator_id = payload["authenticator_id"]
        uv = payload["uv"]
        self.logger.debug(
            "Setting user verified flag on authenticator %s to %s" % (authenticator_id, uv["isUserVerified"]))
        return self.protocol.virtual_authenticator.set_user_verified(authenticator_id, uv)
