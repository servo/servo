import json
import os
import socket
import sys
import threading
import time
import traceback
import urlparse
import uuid

from .base import (Protocol,
                   RefTestExecutor,
                   RefTestImplementation,
                   TestharnessExecutor,
                   extra_timeout,
                   strip_server)
from ..testrunner import Stop

here = os.path.join(os.path.split(__file__)[0])

webdriver = None
exceptions = None
RemoteConnection = None


def do_delayed_imports():
    global webdriver
    global exceptions
    global RemoteConnection
    from selenium import webdriver
    from selenium.common import exceptions
    from selenium.webdriver.remote.remote_connection import RemoteConnection


class SeleniumProtocol(Protocol):
    def __init__(self, executor, browser, capabilities, **kwargs):
        do_delayed_imports()

        Protocol.__init__(self, executor, browser)
        self.capabilities = capabilities
        self.url = browser.webdriver_url
        self.webdriver = None

    def setup(self, runner):
        """Connect to browser via Selenium's WebDriver implementation."""
        self.runner = runner
        self.logger.debug("Connecting to Selenium on URL: %s" % self.url)

        session_started = False
        try:
            self.webdriver = webdriver.Remote(command_executor=RemoteConnection(self.url.strip("/"),
                                                                                resolve_ip=False),
                                              desired_capabilities=self.capabilities)
        except:
            self.logger.warning(
                "Connecting to Selenium failed:\n%s" % traceback.format_exc())
        else:
            self.logger.debug("Selenium session started")
            session_started = True

        if not session_started:
            self.logger.warning("Failed to connect to Selenium")
            self.executor.runner.send_message("init_failed")
        else:
            try:
                self.after_connect()
            except:
                print >> sys.stderr, traceback.format_exc()
                self.logger.warning(
                    "Failed to connect to navigate initial page")
                self.executor.runner.send_message("init_failed")
            else:
                self.executor.runner.send_message("init_succeeded")

    def teardown(self):
        self.logger.debug("Hanging up on Selenium session")
        try:
            self.webdriver.quit()
        except:
            pass
        del self.webdriver

    def is_alive(self):
        try:
            # Get a simple property over the connection
            self.webdriver.current_window_handle
        # TODO what exception?
        except (socket.timeout, exceptions.ErrorInResponseException):
            return False
        return True

    def after_connect(self):
        self.load_runner("http")

    def load_runner(self, protocol):
        url = urlparse.urljoin(self.executor.server_url(protocol),
                               "/testharness_runner.html")
        self.logger.debug("Loading %s" % url)
        self.webdriver.get(url)
        self.webdriver.execute_script("document.title = '%s'" %
                                      threading.current_thread().name.replace("'", '"'))

    def wait(self):
        while True:
            try:
                self.webdriver.execute_async_script("");
            except exceptions.TimeoutException:
                pass
            except (socket.timeout, exceptions.NoSuchWindowException,
                    exceptions.ErrorInResponseException, IOError):
                break
            except Exception as e:
                self.logger.error(traceback.format_exc(e))
                break


class SeleniumRun(object):
    def __init__(self, func, webdriver, url, timeout):
        self.func = func
        self.result = None
        self.webdriver = webdriver
        self.url = url
        self.timeout = timeout
        self.result_flag = threading.Event()

    def run(self):
        timeout = self.timeout

        try:
            self.webdriver.set_script_timeout((timeout + extra_timeout) * 1000)
        except exceptions.ErrorInResponseException:
            self.logger.error("Lost WebDriver connection")
            return Stop

        executor = threading.Thread(target=self._run)
        executor.start()

        flag = self.result_flag.wait(timeout + 2 * extra_timeout)
        if self.result is None:
            assert not flag
            self.result = False, ("EXTERNAL-TIMEOUT", None)

        return self.result

    def _run(self):
        try:
            self.result = True, self.func(self.webdriver, self.url, self.timeout)
        except exceptions.TimeoutException:
            self.result = False, ("EXTERNAL-TIMEOUT", None)
        except (socket.timeout, exceptions.ErrorInResponseException):
            self.result = False, ("CRASH", None)
        except Exception as e:
            message = getattr(e, "message", "")
            if message:
                message += "\n"
            message += traceback.format_exc(e)
            self.result = False, ("ERROR", e)
        finally:
            self.result_flag.set()


class SeleniumTestharnessExecutor(TestharnessExecutor):
    supports_testdriver = True

    def __init__(self, browser, server_config, timeout_multiplier=1,
                 close_after_done=True, capabilities=None, debug_info=None,
                 **kwargs):
        """Selenium-based executor for testharness.js tests"""
        TestharnessExecutor.__init__(self, browser, server_config,
                                     timeout_multiplier=timeout_multiplier,
                                     debug_info=debug_info)
        self.protocol = SeleniumProtocol(self, browser, capabilities)
        with open(os.path.join(here, "testharness_webdriver.js")) as f:
            self.script = f.read()
        with open(os.path.join(here, "testharness_webdriver_resume.js")) as f:
            self.script_resume = f.read()
        self.close_after_done = close_after_done
        self.window_id = str(uuid.uuid4())

    def is_alive(self):
        return self.protocol.is_alive()

    def on_environment_change(self, new_environment):
        if new_environment["protocol"] != self.last_environment["protocol"]:
            self.protocol.load_runner(new_environment["protocol"])

    def do_test(self, test):
        url = self.test_url(test)

        success, data = SeleniumRun(self.do_testharness,
                                    self.protocol.webdriver,
                                    url,
                                    test.timeout * self.timeout_multiplier).run()

        if success:
            return self.convert_result(test, data)

        return (test.result_cls(*data), [])

    def do_testharness(self, webdriver, url, timeout):
        format_map = {"abs_url": url,
                      "url": strip_server(url),
                      "window_id": self.window_id,
                      "timeout_multiplier": self.timeout_multiplier,
                      "timeout": timeout * 1000}

        parent = webdriver.current_window_handle
        handles = [item for item in webdriver.window_handles if item != parent]
        for handle in handles:
            try:
                webdriver.switch_to_window(handle)
                webdriver.close()
            except exceptions.NoSuchWindowException:
                pass
        webdriver.switch_to_window(parent)

        webdriver.execute_script(self.script % format_map)
        try:
            # Try this, it's in Level 1 but nothing supports it yet
            win_s = webdriver.execute_script("return window['%s'];" % self.window_id)
            win_obj = json.loads(win_s)
            test_window = win_obj["window-fcc6-11e5-b4f8-330a88ab9d7f"]
        except:
            after = webdriver.window_handles
            if len(after) == 2:
                test_window = next(iter(set(after) - set([parent])))
            elif after[0] == parent and len(after) > 2:
                # Hope the first one here is the test window
                test_window = after[1]
            else:
                raise Exception("unable to find test window")
        assert test_window != parent

        handler = CallbackHandler(webdriver, test_window, self.logger)
        while True:
            result = webdriver.execute_async_script(
                self.script_resume % format_map)
            done, rv = handler(result)
            if done:
                break
        return rv


class CallbackHandler(object):
    def __init__(self, webdriver, test_window, logger):
        self.webdriver = webdriver
        self.test_window = test_window
        self.logger = logger

    def __call__(self, result):
        self.logger.debug("Got async callback: %s" % result[1])
        try:
            attr = getattr(self, "process_%s" % result[1])
        except AttributeError:
            raise ValueError("Unknown callback type %r" % result[1])
        else:
            return attr(result)

    def process_complete(self, result):
        rv = [result[0]] + result[2]
        return True, rv

    def process_action(self, result):
        parent = self.webdriver.current_window_handle
        try:
            self.webdriver.switch_to.window(self.test_window)
            action = result[2]["action"]
            self.logger.debug("Got action: %s" % action)
            if action == "click":
                selector = result[2]["selector"]
                elements = self.webdriver.find_elements_by_css_selector(selector)
                if len(elements) == 0:
                    raise ValueError("Selector matches no elements")
                elif len(elements) > 1:
                    raise ValueError("Selector matches multiple elements")
                self.logger.debug("Clicking element: %s" % selector)
                try:
                    elements[0].click()
                except (exceptions.ElementNotInteractableException,
                        exceptions.ElementNotVisibleException) as e:
                    self._send_message("complete",
                                       "failure",
                                       e)
                    self.logger.debug("Clicking element failed: %s" % str(e))
                else:
                    self._send_message("complete",
                                       "success")
                    self.logger.debug("Clicking element succeeded")
        finally:
            self.webdriver.switch_to.window(parent)

        return False, None

    def _send_message(self, message_type, status, message=None):
        obj = {
            "type": "testdriver-%s" % str(message_type),
            "status": str(status)
        }
        if message:
            obj["message"] = str(message)
        self.webdriver.execute_script("window.postMessage(%s, '*')" % json.dumps(obj))



class SeleniumRefTestExecutor(RefTestExecutor):
    def __init__(self, browser, server_config, timeout_multiplier=1,
                 screenshot_cache=None, close_after_done=True,
                 debug_info=None, capabilities=None, **kwargs):
        """Selenium WebDriver-based executor for reftests"""
        RefTestExecutor.__init__(self,
                                 browser,
                                 server_config,
                                 screenshot_cache=screenshot_cache,
                                 timeout_multiplier=timeout_multiplier,
                                 debug_info=debug_info)
        self.protocol = SeleniumProtocol(self, browser,
                                         capabilities=capabilities)
        self.implementation = RefTestImplementation(self)
        self.close_after_done = close_after_done
        self.has_window = False

        with open(os.path.join(here, "reftest.js")) as f:
            self.script = f.read()
        with open(os.path.join(here, "reftest-wait_webdriver.js")) as f:
            self.wait_script = f.read()

    def is_alive(self):
        return self.protocol.is_alive()

    def do_test(self, test):
        self.logger.info("Test requires OS-level window focus")

        self.protocol.webdriver.set_window_size(600, 600)

        result = self.implementation.run_test(test)

        return self.convert_result(test, result)

    def screenshot(self, test, viewport_size, dpi):
        # https://github.com/w3c/wptrunner/issues/166
        assert viewport_size is None
        assert dpi is None

        return SeleniumRun(self._screenshot,
                           self.protocol.webdriver,
                           self.test_url(test),
                           test.timeout).run()

    def _screenshot(self, webdriver, url, timeout):
        webdriver.get(url)

        webdriver.execute_async_script(self.wait_script)

        screenshot = webdriver.get_screenshot_as_base64()

        # strip off the data:img/png, part of the url
        if screenshot.startswith("data:image/png;base64,"):
            screenshot = screenshot.split(",", 1)[1]

        return screenshot
