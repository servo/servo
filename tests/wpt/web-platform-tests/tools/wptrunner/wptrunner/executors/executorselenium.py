import json
import os
import socket
import sys
import threading
import time
import traceback
import urlparse
import uuid

from .base import (CallbackHandler,
                   RefTestExecutor,
                   RefTestImplementation,
                   TestharnessExecutor,
                   extra_timeout,
                   strip_server)
from .protocol import (BaseProtocolPart,
                       TestharnessProtocolPart,
                       Protocol,
                       SelectorProtocolPart,
                       ClickProtocolPart,
                       SendKeysProtocolPart,
                       TestDriverProtocolPart)
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


class SeleniumBaseProtocolPart(BaseProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def execute_script(self, script, async=False):
        method = self.webdriver.execute_async_script if async else self.webdriver.execute_script
        return method(script)

    def set_timeout(self, timeout):
        self.webdriver.set_script_timeout(timeout * 1000)

    @property
    def current_window(self):
        return self.webdriver.current_window_handle

    def set_window(self, handle):
        self.webdriver.switch_to_window(handle)

    def wait(self):
        while True:
            try:
                self.webdriver.execute_async_script("")
            except exceptions.TimeoutException:
                pass
            except (socket.timeout, exceptions.NoSuchWindowException,
                    exceptions.ErrorInResponseException, IOError):
                break
            except Exception as e:
                self.logger.error(traceback.format_exc(e))
                break


class SeleniumTestharnessProtocolPart(TestharnessProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def load_runner(self, url_protocol):
        url = urlparse.urljoin(self.parent.executor.server_url(url_protocol),
                               "/testharness_runner.html")
        self.logger.debug("Loading %s" % url)
        self.webdriver.get(url)
        self.webdriver.execute_script("document.title = '%s'" %
                                      threading.current_thread().name.replace("'", '"'))

    def close_old_windows(self):
        exclude = self.webdriver.current_window_handle
        handles = [item for item in self.webdriver.window_handles if item != exclude]
        for handle in handles:
            try:
                self.webdriver.switch_to_window(handle)
                self.webdriver.close()
            except exceptions.NoSuchWindowException:
                pass
        self.webdriver.switch_to_window(exclude)
        return exclude

    def get_test_window(self, window_id, parent):
        test_window = None
        if window_id:
            try:
                # Try this, it's in Level 1 but nothing supports it yet
                win_s = self.webdriver.execute_script("return window['%s'];" % self.window_id)
                win_obj = json.loads(win_s)
                test_window = win_obj["window-fcc6-11e5-b4f8-330a88ab9d7f"]
            except Exception:
                pass

        if test_window is None:
            after = self.webdriver.window_handles
            if len(after) == 2:
                test_window = next(iter(set(after) - set([parent])))
            elif after[0] == parent and len(after) > 2:
                # Hope the first one here is the test window
                test_window = after[1]
            else:
                raise Exception("unable to find test window")

        assert test_window != parent
        return test_window


class SeleniumSelectorProtocolPart(SelectorProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def elements_by_selector(self, selector):
        return self.webdriver.find_elements_by_css_selector(selector)


class SeleniumClickProtocolPart(ClickProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def element(self, element):
        return element.click()

class SeleniumSendKeysProtocolPart(SendKeysProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def send_keys(self, element, keys):
        return element.send_keys(keys)


class SeleniumTestDriverProtocolPart(TestDriverProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def send_message(self, message_type, status, message=None):
        obj = {
            "type": "testdriver-%s" % str(message_type),
            "status": str(status)
        }
        if message:
            obj["message"] = str(message)
        self.webdriver.execute_script("window.postMessage(%s, '*')" % json.dumps(obj))


class SeleniumProtocol(Protocol):
    implements = [SeleniumBaseProtocolPart,
                  SeleniumTestharnessProtocolPart,
                  SeleniumSelectorProtocolPart,
                  SeleniumClickProtocolPart,
                  SeleniumSendKeysProtocolPart,
                  SeleniumTestDriverProtocolPart]

    def __init__(self, executor, browser, capabilities, **kwargs):
        do_delayed_imports()

        super(SeleniumProtocol, self).__init__(executor, browser)
        self.capabilities = capabilities
        self.url = browser.webdriver_url
        self.webdriver = None

    def connect(self):
        """Connect to browser via Selenium's WebDriver implementation."""
        self.logger.debug("Connecting to Selenium on URL: %s" % self.url)

        self.webdriver = webdriver.Remote(command_executor=RemoteConnection(self.url.strip("/"),
                                                                            resolve_ip=False),
                                          desired_capabilities=self.capabilities)

    def after_conect(self):
        pass

    def teardown(self):
        self.logger.debug("Hanging up on Selenium session")
        try:
            self.webdriver.quit()
        except Exception:
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
        self.testharness.load_runner(self.executor.last_environment["protocol"])


class SeleniumRun(object):
    def __init__(self, func, protocol, url, timeout):
        self.func = func
        self.result = None
        self.protocol = protocol
        self.url = url
        self.timeout = timeout
        self.result_flag = threading.Event()

    def run(self):
        timeout = self.timeout

        try:
            self.protocol.base.set_timeout((timeout + extra_timeout))
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
            self.result = True, self.func(self.protocol, self.url, self.timeout)
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
            self.protocol.testharness.load_runner(new_environment["protocol"])

    def do_test(self, test):
        url = self.test_url(test)

        success, data = SeleniumRun(self.do_testharness,
                                    self.protocol,
                                    url,
                                    test.timeout * self.timeout_multiplier).run()

        if success:
            return self.convert_result(test, data)

        return (test.result_cls(*data), [])

    def do_testharness(self, protocol, url, timeout):
        format_map = {"abs_url": url,
                      "url": strip_server(url),
                      "window_id": self.window_id,
                      "timeout_multiplier": self.timeout_multiplier,
                      "timeout": timeout * 1000}

        parent_window = protocol.testharness.close_old_windows()
        # Now start the test harness
        protocol.base.execute_script(self.script % format_map)
        test_window = protocol.testharness.get_test_window(webdriver, parent_window)

        handler = CallbackHandler(self.logger, protocol, test_window)
        while True:
            result = protocol.base.execute_script(
                self.script_resume % format_map, async=True)
            done, rv = handler(result)
            if done:
                break
        return rv


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
                           self.protocol,
                           self.test_url(test),
                           test.timeout).run()

    def _screenshot(self, protocol, url, timeout):
        webdriver = protocol.webdriver
        webdriver.get(url)

        webdriver.execute_async_script(self.wait_script)

        screenshot = webdriver.get_screenshot_as_base64()

        # strip off the data:img/png, part of the url
        if screenshot.startswith("data:image/png;base64,"):
            screenshot = screenshot.split(",", 1)[1]

        return screenshot
