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
                       ActionSequenceProtocolPart,
                       TestDriverProtocolPart,
                       GenerateTestReportProtocolPart)
from ..testrunner import Stop

import webdriver as client

here = os.path.join(os.path.split(__file__)[0])


class WebDriverBaseProtocolPart(BaseProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def execute_script(self, script, asynchronous=False):
        method = self.webdriver.execute_async_script if asynchronous else self.webdriver.execute_script
        return method(script)

    def set_timeout(self, timeout):
        try:
            self.webdriver.timeouts.script = timeout
        except client.WebDriverException:
            # workaround https://bugs.chromium.org/p/chromedriver/issues/detail?id=2057
            body = {"type": "script", "ms": timeout * 1000}
            self.webdriver.send_session_command("POST", "timeouts", body)

    @property
    def current_window(self):
        return self.webdriver.window_handle

    def set_window(self, handle):
        self.webdriver.window_handle = handle

    def wait(self):
        while True:
            try:
                self.webdriver.execute_async_script("")
            except (client.TimeoutException, client.ScriptTimeoutException):
                pass
            except (socket.timeout, client.NoSuchWindowException,
                    client.UnknownErrorException, IOError):
                break
            except Exception as e:
                self.logger.error(traceback.format_exc(e))
                break


class WebDriverTestharnessProtocolPart(TestharnessProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver
        self.runner_handle = None
        with open(os.path.join(here, "runner.js")) as f:
            self.runner_script = f.read()

    def load_runner(self, url_protocol):
        if self.runner_handle:
            self.webdriver.window_handle = self.runner_handle
        url = urlparse.urljoin(self.parent.executor.server_url(url_protocol),
                               "/testharness_runner.html")
        self.logger.debug("Loading %s" % url)

        self.webdriver.url = url
        self.runner_handle = self.webdriver.window_handle
        format_map = {"title": threading.current_thread().name.replace("'", '"')}
        self.parent.base.execute_script(self.runner_script % format_map)

    def close_old_windows(self):
        self.webdriver.actions.release()
        handles = [item for item in self.webdriver.handles if item != self.runner_handle]
        for handle in handles:
            try:
                self.webdriver.window_handle = handle
                self.webdriver.close()
            except client.NoSuchWindowException:
                pass
        self.webdriver.window_handle = self.runner_handle
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
                after = self.webdriver.handles
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


class WebDriverSelectorProtocolPart(SelectorProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def elements_by_selector(self, selector):
        return self.webdriver.find.css(selector)

    def elements_by_selector_and_frame(self, element_selector, frame):
        return self.webdriver.find.css(element_selector, frame=frame)


class WebDriverClickProtocolPart(ClickProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def element(self, element):
        self.logger.info("click " + repr(element))
        return element.click()


class WebDriverSendKeysProtocolPart(SendKeysProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def send_keys(self, element, keys):
        try:
            return element.send_keys(keys)
        except client.UnknownErrorException as e:
            # workaround https://bugs.chromium.org/p/chromedriver/issues/detail?id=1999
            if (e.http_status != 500 or
                e.status_code != "unknown error"):
                raise
            return element.send_element_command("POST", "value", {"value": list(keys)})


class WebDriverActionSequenceProtocolPart(ActionSequenceProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def send_actions(self, actions):
        self.webdriver.actions.perform(actions['actions'])


class WebDriverTestDriverProtocolPart(TestDriverProtocolPart):
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


class WebDriverGenerateTestReportProtocolPart(GenerateTestReportProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def generate_test_report(self, message):
        json_message = {"message": message}
        self.webdriver.send_session_command("POST", "reporting/generate_test_report", json_message)


class WebDriverProtocol(Protocol):
    implements = [WebDriverBaseProtocolPart,
                  WebDriverTestharnessProtocolPart,
                  WebDriverSelectorProtocolPart,
                  WebDriverClickProtocolPart,
                  WebDriverSendKeysProtocolPart,
                  WebDriverActionSequenceProtocolPart,
                  WebDriverTestDriverProtocolPart,
                  WebDriverGenerateTestReportProtocolPart]

    def __init__(self, executor, browser, capabilities, **kwargs):
        super(WebDriverProtocol, self).__init__(executor, browser)
        self.capabilities = capabilities
        self.url = browser.webdriver_url
        self.webdriver = None

    def connect(self):
        """Connect to browser via WebDriver."""
        self.logger.debug("Connecting to WebDriver on URL: %s" % self.url)

        host, port = self.url.split(":")[1].strip("/"), self.url.split(':')[-1].strip("/")

        capabilities = {"alwaysMatch": self.capabilities}
        self.webdriver = client.Session(host, port, capabilities=capabilities)
        self.webdriver.start()


    def teardown(self):
        self.logger.debug("Hanging up on WebDriver session")
        try:
            self.webdriver.end()
        except Exception as e:
            message = str(getattr(e, "message", ""))
            if message:
                message += "\n"
            message += traceback.format_exc(e)
            self.logger.debug(message)
        self.webdriver = None

    def is_alive(self):
        try:
            # Get a simple property over the connection
            self.webdriver.window_handle
        except (socket.timeout, client.UnknownErrorException):
            return False
        return True

    def after_connect(self):
        self.testharness.load_runner(self.executor.last_environment["protocol"])


class WebDriverRun(object):
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
            self.protocol.base.set_timeout(timeout + extra_timeout)
        except client.UnknownErrorException:
            self.logger.error("Lost WebDriver connection")
            return Stop

        executor = threading.Thread(target=self._run)
        executor.start()

        flag = self.result_flag.wait(timeout + 2 * extra_timeout)
        if self.result is None:
            if flag:
                # flag is True unless we timeout; this *shouldn't* happen, but
                # it can if self._run fails to set self.result due to raising
                self.result = False, ("INTERNAL-ERROR", "self._run didn't set a result")
            else:
                message = "Waiting on browser:\n"
                # get a traceback for the current stack of the executor thread
                message += "".join(traceback.format_stack(sys._current_frames()[executor.ident]))
                self.result = False, ("EXTERNAL-TIMEOUT", message)

        return self.result

    def _run(self):
        try:
            self.result = True, self.func(self.protocol, self.url, self.timeout)
        except (client.TimeoutException, client.ScriptTimeoutException):
            self.result = False, ("EXTERNAL-TIMEOUT", None)
        except (socket.timeout, client.UnknownErrorException):
            self.result = False, ("CRASH", None)
        except Exception as e:
            if (isinstance(e, client.WebDriverException) and
                    e.http_status == 408 and
                    e.status_code == "asynchronous script timeout"):
                # workaround for https://bugs.chromium.org/p/chromedriver/issues/detail?id=2001
                self.result = False, ("EXTERNAL-TIMEOUT", None)
            else:
                message = str(getattr(e, "message", ""))
                if message:
                    message += "\n"
                message += traceback.format_exc(e)
                self.result = False, ("INTERNAL-ERROR", message)
        finally:
            self.result_flag.set()


class WebDriverTestharnessExecutor(TestharnessExecutor):
    supports_testdriver = True

    def __init__(self, browser, server_config, timeout_multiplier=1,
                 close_after_done=True, capabilities=None, debug_info=None,
                 supports_eager_pageload=True, **kwargs):
        """WebDriver-based executor for testharness.js tests"""
        TestharnessExecutor.__init__(self, browser, server_config,
                                     timeout_multiplier=timeout_multiplier,
                                     debug_info=debug_info)
        self.protocol = WebDriverProtocol(self, browser, capabilities)
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

        success, data = WebDriverRun(self.do_testharness,
                                    self.protocol,
                                    url,
                                    test.timeout * self.timeout_multiplier).run()

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
        handler = CallbackHandler(self.logger, protocol, test_window)
        protocol.webdriver.url = url

        if not self.supports_eager_pageload:
            self.wait_for_load(protocol)

        while True:
            result = protocol.base.execute_script(
                self.script_resume % format_map, asynchronous=True)

            # As of 2019-03-29, WebDriver does not define expected behavior for
            # cases where the browser crashes during script execution:
            #
            # https://github.com/w3c/webdriver/issues/1308
            if not isinstance(result, list) or len(result) != 2:
                try:
                    is_alive = self.is_alive()
                except client.WebDriverException:
                    is_alive = False

                if not is_alive:
                    raise Exception("Browser crashed during script execution.")

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
            except client.JavascriptErrorException:
                # We can get an error here if the script runs in the initial about:blank
                # document before it has navigated, with the driver returning an error
                # indicating that the document was unloaded
                if seen_error:
                    raise
                seen_error = True


class WebDriverRefTestExecutor(RefTestExecutor):
    def __init__(self, browser, server_config, timeout_multiplier=1,
                 screenshot_cache=None, close_after_done=True,
                 debug_info=None, capabilities=None, **kwargs):
        """WebDriver-based executor for reftests"""
        RefTestExecutor.__init__(self,
                                 browser,
                                 server_config,
                                 screenshot_cache=screenshot_cache,
                                 timeout_multiplier=timeout_multiplier,
                                 debug_info=debug_info)
        self.protocol = WebDriverProtocol(self, browser,
                                          capabilities=capabilities)
        self.implementation = RefTestImplementation(self)
        self.close_after_done = close_after_done
        self.has_window = False

        with open(os.path.join(here, "reftest-wait_webdriver.js")) as f:
            self.wait_script = f.read()

    def reset(self):
        self.implementation.reset()

    def is_alive(self):
        return self.protocol.is_alive()

    def do_test(self, test):
        width_offset, height_offset = self.protocol.webdriver.execute_script(
            """return [window.outerWidth - window.innerWidth,
                       window.outerHeight - window.innerHeight];"""
        )
        try:
            self.protocol.webdriver.window.position = (0, 0)
        except client.InvalidArgumentException:
            # Safari 12 throws with 0 or 1, treating them as bools; fixed in STP
            self.protocol.webdriver.window.position = (2, 2)
        self.protocol.webdriver.window.size = (800 + width_offset, 600 + height_offset)

        result = self.implementation.run_test(test)

        return self.convert_result(test, result)

    def screenshot(self, test, viewport_size, dpi):
        # https://github.com/w3c/wptrunner/issues/166
        assert viewport_size is None
        assert dpi is None

        return WebDriverRun(self._screenshot,
                           self.protocol,
                           self.test_url(test),
                           test.timeout).run()

    def _screenshot(self, protocol, url, timeout):
        webdriver = protocol.webdriver
        webdriver.url = url

        webdriver.execute_async_script(self.wait_script)

        screenshot = webdriver.screenshot()

        # strip off the data:img/png, part of the url
        if screenshot.startswith("data:image/png;base64,"):
            screenshot = screenshot.split(",", 1)[1]

        return screenshot
