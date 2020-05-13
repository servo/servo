import json
import os
import socket
import traceback

from .base import (Protocol,
                   BaseProtocolPart,
                   RefTestExecutor,
                   RefTestImplementation,
                   TestharnessExecutor,
                   TimedRunner,
                   strip_server)
from ..testrunner import Stop
from ..webdriver_server import wait_for_service

webdriver = None
ServoCommandExtensions = None

here = os.path.join(os.path.split(__file__)[0])


def do_delayed_imports():
    global webdriver
    import webdriver

    global ServoCommandExtensions

    class ServoCommandExtensions(object):
        def __init__(self, session):
            self.session = session

        @webdriver.client.command
        def get_prefs(self, *prefs):
            body = {"prefs": list(prefs)}
            return self.session.send_session_command("POST", "servo/prefs/get", body)

        @webdriver.client.command
        def set_prefs(self, prefs):
            body = {"prefs": prefs}
            return self.session.send_session_command("POST", "servo/prefs/set", body)

        @webdriver.client.command
        def reset_prefs(self, *prefs):
            body = {"prefs": list(prefs)}
            return self.session.send_session_command("POST", "servo/prefs/reset", body)

        def change_prefs(self, old_prefs, new_prefs):
            # Servo interprets reset with an empty list as reset everything
            if old_prefs:
                self.reset_prefs(*old_prefs.keys())
            self.set_prefs({k: parse_pref_value(v) for k, v in new_prefs.items()})


# See parse_pref_from_command_line() in components/config/opts.rs
def parse_pref_value(value):
    if value == "true":
        return True
    if value == "false":
        return False
    try:
        return float(value)
    except ValueError:
        return value


class ServoBaseProtocolPart(BaseProtocolPart):
    def execute_script(self, script, asynchronous=False):
        pass

    def set_timeout(self, timeout):
        pass

    def wait(self):
        pass

    def set_window(self, handle):
        pass

    def load(self, url):
        pass


class ServoWebDriverProtocol(Protocol):
    implements = [ServoBaseProtocolPart]

    def __init__(self, executor, browser, capabilities, **kwargs):
        do_delayed_imports()
        Protocol.__init__(self, executor, browser)
        self.capabilities = capabilities
        self.host = browser.webdriver_host
        self.port = browser.webdriver_port
        self.init_timeout = browser.init_timeout
        self.session = None

    def connect(self):
        """Connect to browser via WebDriver."""
        wait_for_service((self.host, self.port), timeout=self.init_timeout)

        self.session = webdriver.Session(self.host, self.port, extension=ServoCommandExtensions)
        self.session.start()

    def after_connect(self):
        pass

    def teardown(self):
        self.logger.debug("Hanging up on WebDriver session")
        try:
            self.session.end()
        except Exception:
            pass

    def is_alive(self):
        try:
            # Get a simple property over the connection
            self.session.window_handle
        # TODO what exception?
        except Exception:
            return False
        return True

    def wait(self):
        while True:
            try:
                self.session.execute_async_script("")
            except webdriver.TimeoutException:
                pass
            except (socket.timeout, IOError):
                break
            except Exception:
                self.logger.error(traceback.format_exc())
                break


class ServoWebDriverRun(TimedRunner):
    def set_timeout(self):
        pass

    def run_func(self):
        try:
            self.result = True, self.func(self.protocol.session, self.url, self.timeout)
        except webdriver.TimeoutException:
            self.result = False, ("EXTERNAL-TIMEOUT", None)
        except (socket.timeout, IOError):
            self.result = False, ("CRASH", None)
        except Exception as e:
            message = getattr(e, "message", "")
            if message:
                message += "\n"
            message += traceback.format_exc()
            self.result = False, ("INTERNAL-ERROR", e)
        finally:
            self.result_flag.set()


class ServoWebDriverTestharnessExecutor(TestharnessExecutor):
    supports_testdriver = True

    def __init__(self, logger, browser, server_config, timeout_multiplier=1,
                 close_after_done=True, capabilities=None, debug_info=None,
                 **kwargs):
        TestharnessExecutor.__init__(self, logger, browser, server_config, timeout_multiplier=1,
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

        timeout = test.timeout * self.timeout_multiplier + self.extra_timeout

        if timeout != self.timeout:
            try:
                self.protocol.session.timeouts.script = timeout
                self.timeout = timeout
            except IOError:
                self.logger.error("Lost webdriver connection")
                return Stop

        success, data = ServoWebDriverRun(self.logger,
                                          self.do_testharness,
                                          self.protocol,
                                          url,
                                          timeout,
                                          self.extra_timeout).run()

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

    def on_environment_change(self, new_environment):
        self.protocol.session.extension.change_prefs(
            self.last_environment.get("prefs", {}),
            new_environment.get("prefs", {})
        )


class TimeoutError(Exception):
    pass


class ServoWebDriverRefTestExecutor(RefTestExecutor):
    def __init__(self, logger, browser, server_config, timeout_multiplier=1,
                 screenshot_cache=None, capabilities=None, debug_info=None,
                 **kwargs):
        """Selenium WebDriver-based executor for reftests"""
        RefTestExecutor.__init__(self,
                                 logger,
                                 browser,
                                 server_config,
                                 screenshot_cache=screenshot_cache,
                                 timeout_multiplier=timeout_multiplier,
                                 debug_info=debug_info)
        self.protocol = ServoWebDriverProtocol(self, browser,
                                               capabilities=capabilities)
        self.implementation = RefTestImplementation(self)
        self.timeout = None
        with open(os.path.join(here, "test-wait.js")) as f:
            self.wait_script = f.read() % {"classname": "reftest-wait"}

    def reset(self):
        self.implementation.reset()

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
            message += traceback.format_exc()
            return test.result_cls("INTERNAL-ERROR", message), []

    def screenshot(self, test, viewport_size, dpi):
        # https://github.com/web-platform-tests/wpt/issues/7135
        assert viewport_size is None
        assert dpi is None

        timeout = (test.timeout * self.timeout_multiplier + self.extra_timeout
                   if self.debug_info is None else None)

        if self.timeout != timeout:
            try:
                self.protocol.session.timeouts.script = timeout
                self.timeout = timeout
            except IOError:
                self.logger.error("Lost webdriver connection")
                return Stop

        return ServoWebDriverRun(self.logger,
                                 self._screenshot,
                                 self.protocol,
                                 self.test_url(test),
                                 timeout,
                                 self.extra_timeout).run()

    def _screenshot(self, session, url, timeout):
        session.url = url
        session.execute_async_script(self.wait_script)
        return session.screenshot()

    def on_environment_change(self, new_environment):
        self.protocol.session.extension.change_prefs(
            self.last_environment.get("prefs", {}),
            new_environment.get("prefs", {})
        )
