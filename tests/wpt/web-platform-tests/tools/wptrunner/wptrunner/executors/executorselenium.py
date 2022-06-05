# mypy: allow-untyped-defs

import json
import os
import socket
import threading
import time
import traceback
import uuid
from urllib.parse import urljoin

from .base import (CallbackHandler,
                   RefTestExecutor,
                   RefTestImplementation,
                   TestharnessExecutor,
                   TimedRunner,
                   strip_server)
from .protocol import (BaseProtocolPart,
                       TestharnessProtocolPart,
                       Protocol,
                       SelectorProtocolPart,
                       ClickProtocolPart,
                       CookiesProtocolPart,
                       SendKeysProtocolPart,
                       WindowProtocolPart,
                       ActionSequenceProtocolPart,
                       TestDriverProtocolPart)

here = os.path.dirname(__file__)

webdriver = None
exceptions = None
RemoteConnection = None
Command = None


def do_delayed_imports():
    global webdriver
    global exceptions
    global RemoteConnection
    global Command
    from selenium import webdriver
    from selenium.common import exceptions
    from selenium.webdriver.remote.remote_connection import RemoteConnection
    from selenium.webdriver.remote.command import Command


class SeleniumBaseProtocolPart(BaseProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def execute_script(self, script, asynchronous=False):
        method = self.webdriver.execute_async_script if asynchronous else self.webdriver.execute_script
        return method(script)

    def set_timeout(self, timeout):
        self.webdriver.set_script_timeout(timeout * 1000)

    @property
    def current_window(self):
        return self.webdriver.current_window_handle

    def set_window(self, handle):
        self.webdriver.switch_to_window(handle)

    def window_handles(self):
        return self.webdriver.window_handles

    def load(self, url):
        self.webdriver.get(url)

    def wait(self):
        while True:
            try:
                return self.webdriver.execute_async_script("""let callback = arguments[arguments.length - 1];
addEventListener("__test_restart", e => {e.preventDefault(); callback(true)})""")
            except exceptions.TimeoutException:
                pass
            except (socket.timeout, exceptions.NoSuchWindowException, exceptions.ErrorInResponseException, OSError):
                break
            except Exception:
                self.logger.error(traceback.format_exc())
                break
        return False


class SeleniumTestharnessProtocolPart(TestharnessProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver
        self.runner_handle = None
        with open(os.path.join(here, "runner.js")) as f:
            self.runner_script = f.read()
        with open(os.path.join(here, "window-loaded.js")) as f:
            self.window_loaded_script = f.read()

    def load_runner(self, url_protocol):
        if self.runner_handle:
            self.webdriver.switch_to_window(self.runner_handle)
        url = urljoin(self.parent.executor.server_url(url_protocol),
                      "/testharness_runner.html")
        self.logger.debug("Loading %s" % url)
        self.webdriver.get(url)
        self.runner_handle = self.webdriver.current_window_handle
        format_map = {"title": threading.current_thread().name.replace("'", '"')}
        self.parent.base.execute_script(self.runner_script % format_map)

    def close_old_windows(self):
        handles = [item for item in self.webdriver.window_handles if item != self.runner_handle]
        for handle in handles:
            try:
                self.webdriver.switch_to_window(handle)
                self.webdriver.close()
            except exceptions.NoSuchWindowException:
                pass
        self.webdriver.switch_to_window(self.runner_handle)
        return self.runner_handle

    def get_test_window(self, window_id, parent, timeout=5):
        """Find the test window amongst all the open windows.
        This is assumed to be either the named window or the one after the parent in the list of
        window handles

        :param window_id: The DOM name of the Window
        :param parent: The handle of the runner window
        :param timeout: The time in seconds to wait for the window to appear. This is because in
                        some implementations there's a race between calling window.open and the
                        window being added to the list of WebDriver accessible windows."""
        test_window = None
        end_time = time.time() + timeout
        while time.time() < end_time:
            try:
                # Try using the JSON serialization of the WindowProxy object,
                # it's in Level 1 but nothing supports it yet
                win_s = self.webdriver.execute_script("return window['%s'];" % window_id)
                win_obj = json.loads(win_s)
                test_window = win_obj["window-fcc6-11e5-b4f8-330a88ab9d7f"]
            except Exception:
                pass

            if test_window is None:
                after = self.webdriver.window_handles
                if len(after) == 2:
                    test_window = next(iter(set(after) - {parent}))
                elif after[0] == parent and len(after) > 2:
                    # Hope the first one here is the test window
                    test_window = after[1]

            if test_window is not None:
                assert test_window != parent
                return test_window

            time.sleep(0.1)

        raise Exception("unable to find test window")

    def test_window_loaded(self):
        """Wait until the page in the new window has been loaded.

        Hereby ignore Javascript execptions that are thrown when
        the document has been unloaded due to a process change.
        """
        while True:
            try:
                self.webdriver.execute_async_script(self.window_loaded_script)
                break
            except exceptions.JavascriptException:
                pass


class SeleniumSelectorProtocolPart(SelectorProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def elements_by_selector(self, selector):
        return self.webdriver.find_elements_by_css_selector(selector)

    def elements_by_selector_and_frame(self, element_selector, frame):
        return self.webdriver.find_elements_by_css_selector(element_selector)


class SeleniumClickProtocolPart(ClickProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def element(self, element):
        return element.click()


class SeleniumCookiesProtocolPart(CookiesProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def delete_all_cookies(self):
        self.logger.info("Deleting all cookies")
        return self.webdriver.delete_all_cookies()

class SeleniumWindowProtocolPart(WindowProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def minimize(self):
        self.previous_rect = self.webdriver.window.rect
        self.logger.info("Minimizing")
        return self.webdriver.minimize()

    def set_rect(self, rect):
        self.logger.info("Setting window rect")
        self.webdriver.window.rect = rect

class SeleniumSendKeysProtocolPart(SendKeysProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def send_keys(self, element, keys):
        return element.send_keys(keys)


class SeleniumActionSequenceProtocolPart(ActionSequenceProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def send_actions(self, actions):
        self.webdriver.execute(Command.W3C_ACTIONS, {"actions": actions})


class SeleniumTestDriverProtocolPart(TestDriverProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def send_message(self, cmd_id, message_type, status, message=None):
        obj = {
            "cmd_id": cmd_id,
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
                  SeleniumCookiesProtocolPart,
                  SeleniumSendKeysProtocolPart,
                  SeleniumTestDriverProtocolPart,
                  SeleniumWindowProtocolPart,
                  SeleniumActionSequenceProtocolPart]

    def __init__(self, executor, browser, capabilities, **kwargs):
        do_delayed_imports()

        super().__init__(executor, browser)
        self.capabilities = capabilities
        self.url = browser.webdriver_url
        self.webdriver = None

    def connect(self):
        """Connect to browser via Selenium's WebDriver implementation."""
        self.logger.debug("Connecting to Selenium on URL: %s" % self.url)

        self.webdriver = webdriver.Remote(command_executor=RemoteConnection(self.url.strip("/"),
                                                                            resolve_ip=False),
                                          desired_capabilities=self.capabilities)

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


class SeleniumRun(TimedRunner):
    def set_timeout(self):
        timeout = self.timeout

        try:
            self.protocol.base.set_timeout(timeout + self.extra_timeout)
        except exceptions.ErrorInResponseException:
            msg = "Lost WebDriver connection"
            self.logger.error(msg)
            return ("INTERNAL-ERROR", msg)

    def run_func(self):
        try:
            self.result = True, self.func(self.protocol, self.url, self.timeout)
        except exceptions.TimeoutException:
            self.result = False, ("EXTERNAL-TIMEOUT", None)
        except (socket.timeout, exceptions.ErrorInResponseException):
            self.result = False, ("CRASH", None)
        except Exception as e:
            message = str(getattr(e, "message", ""))
            if message:
                message += "\n"
            message += traceback.format_exc()
            self.result = False, ("INTERNAL-ERROR", message)
        finally:
            self.result_flag.set()


class SeleniumTestharnessExecutor(TestharnessExecutor):
    supports_testdriver = True

    def __init__(self, logger, browser, server_config, timeout_multiplier=1,
                 close_after_done=True, capabilities=None, debug_info=None,
                 supports_eager_pageload=True, **kwargs):
        """Selenium-based executor for testharness.js tests"""
        TestharnessExecutor.__init__(self, logger, browser, server_config,
                                     timeout_multiplier=timeout_multiplier,
                                     debug_info=debug_info)
        self.protocol = SeleniumProtocol(self, browser, capabilities)
        with open(os.path.join(here, "testharness_webdriver_resume.js")) as f:
            self.script_resume = f.read()
        self.close_after_done = close_after_done
        self.window_id = str(uuid.uuid4())
        self.supports_eager_pageload = supports_eager_pageload

    def is_alive(self):
        return self.protocol.is_alive()

    def on_environment_change(self, new_environment):
        if new_environment["protocol"] != self.last_environment["protocol"]:
            self.protocol.testharness.load_runner(new_environment["protocol"])

    def do_test(self, test):
        url = self.test_url(test)

        success, data = SeleniumRun(self.logger,
                                    self.do_testharness,
                                    self.protocol,
                                    url,
                                    test.timeout * self.timeout_multiplier,
                                    self.extra_timeout).run()

        if success:
            return self.convert_result(test, data)

        return (test.result_cls(*data), [])

    def do_testharness(self, protocol, url, timeout):
        format_map = {"url": strip_server(url)}

        parent_window = protocol.testharness.close_old_windows()
        # Now start the test harness
        protocol.base.execute_script("window.open('about:blank', '%s', 'noopener')" % self.window_id)
        test_window = protocol.testharness.get_test_window(self.window_id,
                                                           parent_window,
                                                           timeout=5*self.timeout_multiplier)
        self.protocol.base.set_window(test_window)
        protocol.testharness.test_window_loaded()

        protocol.base.load(url)

        if not self.supports_eager_pageload:
            self.wait_for_load(protocol)

        handler = CallbackHandler(self.logger, protocol, test_window)
        while True:
            result = protocol.base.execute_script(
                self.script_resume % format_map, asynchronous=True)
            done, rv = handler(result)
            if done:
                break
        return rv

    def wait_for_load(self, protocol):
        # pageLoadStrategy=eager doesn't work in Chrome so try to emulate in user script
        loaded = False
        seen_error = False
        while not loaded:
            try:
                loaded = protocol.base.execute_script("""
var callback = arguments[arguments.length - 1];
if (location.href === "about:blank") {
  callback(false);
} else if (document.readyState !== "loading") {
  callback(true);
} else {
  document.addEventListener("readystatechange", () => {if (document.readyState !== "loading") {callback(true)}});
}""", asynchronous=True)
            except Exception:
                # We can get an error here if the script runs in the initial about:blank
                # document before it has navigated, with the driver returning an error
                # indicating that the document was unloaded
                if seen_error:
                    raise
                seen_error = True


class SeleniumRefTestExecutor(RefTestExecutor):
    def __init__(self, logger, browser, server_config, timeout_multiplier=1,
                 screenshot_cache=None, close_after_done=True,
                 debug_info=None, capabilities=None, **kwargs):
        """Selenium WebDriver-based executor for reftests"""
        RefTestExecutor.__init__(self,
                                 logger,
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

        with open(os.path.join(here, "test-wait.js")) as f:
            self.wait_script = f.read() % {"classname": "reftest-wait"}

    def reset(self):
        self.implementation.reset()

    def is_alive(self):
        return self.protocol.is_alive()

    def do_test(self, test):
        self.logger.info("Test requires OS-level window focus")

        width_offset, height_offset = self.protocol.webdriver.execute_script(
            """return [window.outerWidth - window.innerWidth,
                       window.outerHeight - window.innerHeight];"""
        )
        self.protocol.webdriver.set_window_position(0, 0)
        self.protocol.webdriver.set_window_size(800 + width_offset, 600 + height_offset)

        result = self.implementation.run_test(test)

        return self.convert_result(test, result)

    def screenshot(self, test, viewport_size, dpi, page_ranges):
        # https://github.com/web-platform-tests/wpt/issues/7135
        assert viewport_size is None
        assert dpi is None

        return SeleniumRun(self.logger,
                           self._screenshot,
                           self.protocol,
                           self.test_url(test),
                           test.timeout,
                           self.extra_timeout).run()

    def _screenshot(self, protocol, url, timeout):
        webdriver = protocol.webdriver
        webdriver.get(url)

        webdriver.execute_async_script(self.wait_script)

        screenshot = webdriver.get_screenshot_as_base64()

        # strip off the data:img/png, part of the url
        if screenshot.startswith("data:image/png;base64,"):
            screenshot = screenshot.split(",", 1)[1]

        return screenshot
