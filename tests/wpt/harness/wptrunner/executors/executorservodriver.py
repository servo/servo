# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import json
import os
import socket
import threading
import time
import traceback

from .base import (Protocol,
                   RefTestExecutor,
                   RefTestImplementation,
                   TestharnessExecutor,
                   strip_server)
import webdriver
from ..testrunner import Stop

here = os.path.join(os.path.split(__file__)[0])

extra_timeout = 5


class ServoWebDriverProtocol(Protocol):
    def __init__(self, executor, browser, capabilities, **kwargs):
        Protocol.__init__(self, executor, browser)
        self.capabilities = capabilities
        self.host = browser.webdriver_host
        self.port = browser.webdriver_port
        self.session = None

    def setup(self, runner):
        """Connect to browser via WebDriver."""
        self.runner = runner

        session_started = False
        try:
            self.session = webdriver.Session(self.host, self.port,
                                             extension=webdriver.ServoExtensions)
            self.session.start()
        except:
            self.logger.warning(
                "Connecting with WebDriver failed:\n%s" % traceback.format_exc())
        else:
            self.logger.debug("session started")
            session_started = True

        if not session_started:
            self.logger.warning("Failed to connect via WebDriver")
            self.executor.runner.send_message("init_failed")
        else:
            self.executor.runner.send_message("init_succeeded")

    def teardown(self):
        self.logger.debug("Hanging up on WebDriver session")
        try:
            self.session.end()
        except:
            pass

    def is_alive(self):
        try:
            # Get a simple property over the connection
            self.session.handle
        # TODO what exception?
        except Exception:
            return False
        return True

    def after_connect(self):
        pass

    def wait(self):
        while True:
            try:
                self.session.execute_async_script("")
            except webdriver.TimeoutException:
                pass
            except (socket.timeout, IOError):
                break
            except Exception as e:
                self.logger.error(traceback.format_exc(e))
                break

    def on_environment_change(self, old_environment, new_environment):
        #Unset all the old prefs
        self.session.extension.reset_prefs(*old_environment.get("prefs", {}).keys())
        self.session.extension.set_prefs(new_environment.get("prefs", {}))


class ServoWebDriverRun(object):
    def __init__(self, func, session, url, timeout, current_timeout=None):
        self.func = func
        self.result = None
        self.session = session
        self.url = url
        self.timeout = timeout
        self.result_flag = threading.Event()

    def run(self):
        executor = threading.Thread(target=self._run)
        executor.start()

        flag = self.result_flag.wait(self.timeout + extra_timeout)
        if self.result is None:
            assert not flag
            self.result = False, ("EXTERNAL-TIMEOUT", None)

        return self.result

    def _run(self):
        try:
            self.result = True, self.func(self.session, self.url, self.timeout)
        except webdriver.TimeoutException:
            self.result = False, ("EXTERNAL-TIMEOUT", None)
        except (socket.timeout, IOError):
            self.result = False, ("CRASH", None)
        except Exception as e:
            message = getattr(e, "message", "")
            if message:
                message += "\n"
            message += traceback.format_exc(e)
            self.result = False, ("ERROR", e)
        finally:
            self.result_flag.set()


def timeout_func(timeout):
    if timeout:
        t0 = time.time()
        return lambda: time.time() - t0 > timeout + extra_timeout
    else:
        return lambda: False


class ServoWebDriverTestharnessExecutor(TestharnessExecutor):
    def __init__(self, browser, server_config, timeout_multiplier=1,
                 close_after_done=True, capabilities=None, debug_info=None):
        TestharnessExecutor.__init__(self, browser, server_config, timeout_multiplier=1,
                                     debug_info=None)
        self.protocol = ServoWebDriverProtocol(self, browser, capabilities=capabilities)
        with open(os.path.join(here, "testharness_servodriver.js")) as f:
            self.script = f.read()
        self.timeout = None

    def on_protocol_change(self, new_protocol):
        pass

    def is_alive(self):
        return self.protocol.is_alive()

    def do_test(self, test):
        url = self.test_url(test)

        timeout = test.timeout * self.timeout_multiplier + extra_timeout

        if timeout != self.timeout:
            try:
                self.protocol.session.timeouts.script = timeout
                self.timeout = timeout
            except IOError:
                self.logger.error("Lost webdriver connection")
                return Stop

        success, data = ServoWebDriverRun(self.do_testharness,
                                          self.protocol.session,
                                          url,
                                          timeout).run()

        if success:
            return self.convert_result(test, data)

        return (test.result_cls(*data), [])

    def do_testharness(self, session, url, timeout):
        session.url = url
        result = json.loads(
            session.execute_async_script(
                self.script % {"abs_url": url,
                               "url": strip_server(url),
                               "timeout_multiplier": self.timeout_multiplier,
                               "timeout": timeout * 1000}))
        # Prevent leaking every page in history until Servo develops a more sane
        # page cache
        session.back()
        return result


class TimeoutError(Exception):
    pass


class ServoWebDriverRefTestExecutor(RefTestExecutor):
    def __init__(self, browser, server_config, timeout_multiplier=1,
                 screenshot_cache=None, capabilities=None, debug_info=None):
        """Selenium WebDriver-based executor for reftests"""
        RefTestExecutor.__init__(self,
                                 browser,
                                 server_config,
                                 screenshot_cache=screenshot_cache,
                                 timeout_multiplier=timeout_multiplier,
                                 debug_info=debug_info)
        self.protocol = ServoWebDriverProtocol(self, browser,
                                               capabilities=capabilities)
        self.implementation = RefTestImplementation(self)
        self.timeout = None
        with open(os.path.join(here, "reftest-wait_servodriver.js")) as f:
            self.wait_script = f.read()

    def is_alive(self):
        return self.protocol.is_alive()

    def do_test(self, test):
        try:
            result = self.implementation.run_test(test)
            return self.convert_result(test, result)
        except IOError:
            return test.result_cls("CRASH", None), []
        except TimeoutError:
            return test.result_cls("TIMEOUT", None), []
        except Exception as e:
            message = getattr(e, "message", "")
            if message:
                message += "\n"
            message += traceback.format_exc(e)
            return test.result_cls("ERROR", message), []

    def screenshot(self, test):
        timeout = (test.timeout * self.timeout_multiplier + extra_timeout
                   if self.debug_info is None else None)

        if self.timeout != timeout:
            try:
                self.protocol.session.timeouts.script = timeout
                self.timeout = timeout
            except IOError:
                self.logger.error("Lost webdriver connection")
                return Stop

        return ServoWebDriverRun(self._screenshot,
                                 self.protocol.session,
                                 self.test_url(test),
                                 timeout).run()

    def _screenshot(self, session, url, timeout):
        session.url = url
        session.execute_async_script(self.wait_script)
        return session.screenshot()
