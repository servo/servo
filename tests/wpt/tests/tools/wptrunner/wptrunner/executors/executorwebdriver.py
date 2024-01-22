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
                   CrashtestExecutor,
                   RefTestExecutor,
                   RefTestImplementation,
                   TestharnessExecutor,
                   TimedRunner,
                   strip_server)
from .protocol import (BaseProtocolPart,
                       TestharnessProtocolPart,
                       Protocol,
                       SelectorProtocolPart,
                       AccessibilityProtocolPart,
                       ClickProtocolPart,
                       CookiesProtocolPart,
                       SendKeysProtocolPart,
                       ActionSequenceProtocolPart,
                       TestDriverProtocolPart,
                       GenerateTestReportProtocolPart,
                       SetPermissionProtocolPart,
                       VirtualAuthenticatorProtocolPart,
                       WindowProtocolPart,
                       DebugProtocolPart,
                       SPCTransactionsProtocolPart,
                       RPHRegistrationsProtocolPart,
                       FedCMProtocolPart,
                       VirtualSensorProtocolPart,
                       merge_dicts)

from webdriver.client import Session
from webdriver import error

here = os.path.dirname(__file__)


class WebDriverCallbackHandler(CallbackHandler):
    unimplemented_exc = (NotImplementedError, error.UnknownCommandException)
    expected_exc = (error.WebDriverException,)


class WebDriverBaseProtocolPart(BaseProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def execute_script(self, script, asynchronous=False, args=None):
        method = self.webdriver.execute_async_script if asynchronous else self.webdriver.execute_script
        return method(script, args=args)

    def set_timeout(self, timeout):
        try:
            self.webdriver.timeouts.script = timeout
        except error.WebDriverException:
            # workaround https://bugs.chromium.org/p/chromedriver/issues/detail?id=2057
            body = {"type": "script", "ms": timeout * 1000}
            self.webdriver.send_session_command("POST", "timeouts", body)

    @property
    def current_window(self):
        return self.webdriver.window_handle

    def set_window(self, handle):
        self.webdriver.window_handle = handle

    def window_handles(self):
        return self.webdriver.handles

    def load(self, url):
        self.webdriver.url = url

    def wait(self):
        while True:
            try:
                self.webdriver.execute_async_script("""let callback = arguments[arguments.length - 1];
addEventListener("__test_restart", e => {e.preventDefault(); callback(true)})""")
                self.webdriver.execute_async_script("")
            except (error.TimeoutException,
                    error.ScriptTimeoutException,
                    error.JavascriptErrorException):
                # A JavascriptErrorException will happen when we navigate;
                # by ignoring it it's possible to reload the test whilst the
                # harness remains paused
                pass
            except (socket.timeout, error.NoSuchWindowException, error.UnknownErrorException, OSError):
                break
            except Exception:
                message = "Uncaught exception in WebDriverBaseProtocolPart.wait:\n"
                message += traceback.format_exc()
                self.logger.error(message)
                break
        return False


class WebDriverTestharnessProtocolPart(TestharnessProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver
        self.runner_handle = None
        with open(os.path.join(here, "runner.js")) as f:
            self.runner_script = f.read()
        with open(os.path.join(here, "window-loaded.js")) as f:
            self.window_loaded_script = f.read()

    def load_runner(self, url_protocol):
        if self.runner_handle:
            self.webdriver.window_handle = self.runner_handle
        url = urljoin(self.parent.executor.server_url(url_protocol),
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
            self._close_window(handle)
        self.webdriver.window_handle = self.runner_handle
        return self.runner_handle

    def _close_window(self, window_handle):
        try:
            self.webdriver.window_handle = window_handle
            self.webdriver.window.close()
        except error.NoSuchWindowException:
            pass

    def open_test_window(self, window_id):
        self.webdriver.execute_script(
            "window.open('about:blank', '%s', 'noopener')" % window_id)

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
                test_window = self._poll_handles_for_test_window(parent)

            if test_window is not None:
                assert test_window != parent
                return test_window

            time.sleep(0.1)

        raise Exception("unable to find test window")

    def _poll_handles_for_test_window(self, parent):
        test_window = None
        after = self.webdriver.handles
        if len(after) == 2:
            test_window = next(iter(set(after) - {parent}))
        elif after[0] == parent and len(after) > 2:
            # Hope the first one here is the test window
            test_window = after[1]
        return test_window

    def test_window_loaded(self):
        """Wait until the page in the new window has been loaded.

        Hereby ignore Javascript execptions that are thrown when
        the document has been unloaded due to a process change.
        """
        while True:
            try:
                self.webdriver.execute_script(self.window_loaded_script, asynchronous=True)
                break
            except error.JavascriptErrorException:
                pass


class WebDriverSelectorProtocolPart(SelectorProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def elements_by_selector(self, selector):
        return self.webdriver.find.css(selector)


class WebDriverAccessibilityProtocolPart(AccessibilityProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def get_computed_label(self, element):
        return element.get_computed_label()

    def get_computed_role(self, element):
        return element.get_computed_role()


class WebDriverClickProtocolPart(ClickProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def element(self, element):
        self.logger.info("click " + repr(element))
        return element.click()


class WebDriverCookiesProtocolPart(CookiesProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def delete_all_cookies(self):
        self.logger.info("Deleting all cookies")
        return self.webdriver.send_session_command("DELETE", "cookie")

    def get_all_cookies(self):
        self.logger.info("Getting all cookies")
        return self.webdriver.send_session_command("GET", "cookie")

    def get_named_cookie(self, name):
        self.logger.info("Getting cookie named %s" % name)
        try:
            return self.webdriver.send_session_command("GET", "cookie/%s" % name)
        except error.NoSuchCookieException:
            return None


class WebDriverWindowProtocolPart(WindowProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def minimize(self):
        self.logger.info("Minimizing")
        return self.webdriver.window.minimize()

    def set_rect(self, rect):
        self.logger.info("Restoring")
        self.webdriver.window.rect = rect

    def get_rect(self):
        self.logger.info("Getting rect")
        return self.webdriver.window.rect

class WebDriverSendKeysProtocolPart(SendKeysProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def send_keys(self, element, keys):
        try:
            return element.send_keys(keys)
        except error.UnknownErrorException as e:
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

    def release(self):
        self.webdriver.actions.release()


class WebDriverTestDriverProtocolPart(TestDriverProtocolPart):
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

    def _switch_to_frame(self, index_or_elem):
        try:
            self.webdriver.switch_frame(index_or_elem)
        except (error.StaleElementReferenceException,
                error.NoSuchFrameException) as e:
            raise ValueError from e

    def _switch_to_parent_frame(self):
        self.webdriver.switch_frame("parent")


class WebDriverGenerateTestReportProtocolPart(GenerateTestReportProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def generate_test_report(self, message):
        json_message = {"message": message}
        self.webdriver.send_session_command("POST", "reporting/generate_test_report", json_message)


class WebDriverSetPermissionProtocolPart(SetPermissionProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def set_permission(self, descriptor, state):
        permission_params_dict = {
            "descriptor": descriptor,
            "state": state,
        }
        self.webdriver.send_session_command("POST", "permissions", permission_params_dict)


class WebDriverVirtualAuthenticatorProtocolPart(VirtualAuthenticatorProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def add_virtual_authenticator(self, config):
        return self.webdriver.send_session_command("POST", "webauthn/authenticator", config)

    def remove_virtual_authenticator(self, authenticator_id):
        return self.webdriver.send_session_command("DELETE", "webauthn/authenticator/%s" % authenticator_id)

    def add_credential(self, authenticator_id, credential):
        return self.webdriver.send_session_command("POST", "webauthn/authenticator/%s/credential" % authenticator_id, credential)

    def get_credentials(self, authenticator_id):
        return self.webdriver.send_session_command("GET", "webauthn/authenticator/%s/credentials" % authenticator_id)

    def remove_credential(self, authenticator_id, credential_id):
        return self.webdriver.send_session_command("DELETE", f"webauthn/authenticator/{authenticator_id}/credentials/{credential_id}")

    def remove_all_credentials(self, authenticator_id):
        return self.webdriver.send_session_command("DELETE", "webauthn/authenticator/%s/credentials" % authenticator_id)

    def set_user_verified(self, authenticator_id, uv):
        return self.webdriver.send_session_command("POST", "webauthn/authenticator/%s/uv" % authenticator_id, uv)


class WebDriverSPCTransactionsProtocolPart(SPCTransactionsProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def set_spc_transaction_mode(self, mode):
        body = {"mode": mode}
        return self.webdriver.send_session_command("POST", "secure-payment-confirmation/set-mode", body)

class WebDriverRPHRegistrationsProtocolPart(RPHRegistrationsProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def set_rph_registration_mode(self, mode):
        body = {"mode": mode}
        return self.webdriver.send_session_command("POST", "custom-handlers/set-mode", body)

class WebDriverFedCMProtocolPart(FedCMProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def cancel_fedcm_dialog(self):
        return self.webdriver.send_session_command("POST", "fedcm/canceldialog")

    def click_fedcm_dialog_button(self, dialog_button):
        body = {"dialogButton": dialog_button}
        return self.webdriver.send_session_command("POST", "fedcm/clickdialogbutton", body)

    def select_fedcm_account(self, account_index):
        body = {"accountIndex": account_index}
        return self.webdriver.send_session_command("POST", "fedcm/selectaccount", body)

    def get_fedcm_account_list(self):
        return self.webdriver.send_session_command("GET", "fedcm/accountlist")

    def get_fedcm_dialog_title(self):
        return self.webdriver.send_session_command("GET", "fedcm/gettitle")

    def get_fedcm_dialog_type(self):
        return self.webdriver.send_session_command("GET", "fedcm/getdialogtype")

    def set_fedcm_delay_enabled(self, enabled):
        body = {"enabled": enabled}
        return self.webdriver.send_session_command("POST", "fedcm/setdelayenabled", body)

    def reset_fedcm_cooldown(self):
        return self.webdriver.send_session_command("POST", "fedcm/resetcooldown")


class WebDriverDebugProtocolPart(DebugProtocolPart):
    def load_devtools(self):
        raise NotImplementedError()


class WebDriverVirtualSensorPart(VirtualSensorProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def create_virtual_sensor(self, sensor_type, sensor_params):
        body = {"type": sensor_type}
        body.update(sensor_params)
        return self.webdriver.send_session_command("POST", "sensor", body)

    def update_virtual_sensor(self, sensor_type, reading):
        body = {"reading": reading}
        return self.webdriver.send_session_command("POST", "sensor/%s" % sensor_type, body)

    def remove_virtual_sensor(self, sensor_type):
        return self.webdriver.send_session_command("DELETE", "sensor/%s" % sensor_type)

    def get_virtual_sensor_information(self, sensor_type):
        return self.webdriver.send_session_command("GET", "sensor/%s" % sensor_type)


class WebDriverProtocol(Protocol):
    implements = [WebDriverBaseProtocolPart,
                  WebDriverTestharnessProtocolPart,
                  WebDriverSelectorProtocolPart,
                  WebDriverAccessibilityProtocolPart,
                  WebDriverClickProtocolPart,
                  WebDriverCookiesProtocolPart,
                  WebDriverSendKeysProtocolPart,
                  WebDriverWindowProtocolPart,
                  WebDriverActionSequenceProtocolPart,
                  WebDriverTestDriverProtocolPart,
                  WebDriverGenerateTestReportProtocolPart,
                  WebDriverSetPermissionProtocolPart,
                  WebDriverVirtualAuthenticatorProtocolPart,
                  WebDriverSPCTransactionsProtocolPart,
                  WebDriverRPHRegistrationsProtocolPart,
                  WebDriverFedCMProtocolPart,
                  WebDriverDebugProtocolPart,
                  WebDriverVirtualSensorPart]

    def __init__(self, executor, browser, capabilities, **kwargs):
        super().__init__(executor, browser)
        self.capabilities = capabilities
        if hasattr(browser, "capabilities"):
            if self.capabilities is None:
                self.capabilities = browser.capabilities
            else:
                merge_dicts(self.capabilities, browser.capabilities)

        pac = browser.pac
        if pac is not None:
            if self.capabilities is None:
                self.capabilities = {}
            merge_dicts(self.capabilities, {"proxy":
                {
                    "proxyType": "pac",
                    "proxyAutoconfigUrl": urljoin(executor.server_url("http"), pac)
                }
            })

        self.url = browser.webdriver_url
        self.webdriver = None

    def connect(self):
        """Connect to browser via WebDriver."""
        self.logger.debug("Connecting to WebDriver on URL: %s" % self.url)

        host, port = self.url.split(":")[1].strip("/"), self.url.split(':')[-1].strip("/")

        capabilities = {"alwaysMatch": self.capabilities}
        self.webdriver = Session(host, port, capabilities=capabilities)
        self.webdriver.start()

    def teardown(self):
        self.logger.debug("Hanging up on WebDriver session")
        try:
            self.webdriver.end()
        except Exception as e:
            message = str(getattr(e, "message", ""))
            if message:
                message += "\n"
            message += traceback.format_exc()
            self.logger.debug(message)
        self.webdriver = None

    def is_alive(self):
        try:
            # Get a simple property over the connection, with 2 seconds of timeout
            # that should be more than enough to check if the WebDriver its
            # still alive, and allows to complete the check within the testrunner
            # 5 seconds of extra_timeout we have as maximum to end the test before
            # the external timeout from testrunner triggers.
            self.webdriver.send_session_command("GET", "window", timeout=2)
        except (socket.timeout, error.UnknownErrorException, error.InvalidSessionIdException):
            return False
        return True

    def after_connect(self):
        self.testharness.load_runner(self.executor.last_environment["protocol"])


class WebDriverRun(TimedRunner):
    def set_timeout(self):
        try:
            self.protocol.base.set_timeout(self.timeout + self.extra_timeout)
        except error.UnknownErrorException:
            msg = "Lost WebDriver connection"
            self.logger.error(msg)
            return ("INTERNAL-ERROR", msg)

    def run_func(self):
        try:
            self.result = True, self.func(self.protocol, self.url, self.timeout)
        except (error.TimeoutException, error.ScriptTimeoutException):
            self.result = False, ("EXTERNAL-TIMEOUT", None)
        except (socket.timeout, error.UnknownErrorException):
            self.result = False, ("CRASH", None)
        except Exception as e:
            if (isinstance(e, error.WebDriverException) and
                    e.http_status == 408 and
                    e.status_code == "asynchronous script timeout"):
                # workaround for https://bugs.chromium.org/p/chromedriver/issues/detail?id=2001
                self.result = False, ("EXTERNAL-TIMEOUT", None)
            else:
                message = str(getattr(e, "message", ""))
                if message:
                    message += "\n"
                message += traceback.format_exc()
                self.result = False, ("INTERNAL-ERROR", message)
        finally:
            self.result_flag.set()


class WebDriverTestharnessExecutor(TestharnessExecutor):
    supports_testdriver = True
    protocol_cls = WebDriverProtocol

    def __init__(self, logger, browser, server_config, timeout_multiplier=1,
                 close_after_done=True, capabilities=None, debug_info=None,
                 cleanup_after_test=True, **kwargs):
        """WebDriver-based executor for testharness.js tests"""
        TestharnessExecutor.__init__(self, logger, browser, server_config,
                                     timeout_multiplier=timeout_multiplier,
                                     debug_info=debug_info)
        self.protocol = self.protocol_cls(self, browser, capabilities)
        with open(os.path.join(here, "testharness_webdriver_resume.js")) as f:
            self.script_resume = f.read()
        with open(os.path.join(here, "window-loaded.js")) as f:
            self.window_loaded_script = f.read()

        self.close_after_done = close_after_done
        self.window_id = str(uuid.uuid4())
        self.cleanup_after_test = cleanup_after_test

    def is_alive(self):
        return self.protocol.is_alive()

    def on_environment_change(self, new_environment):
        if new_environment["protocol"] != self.last_environment["protocol"]:
            self.protocol.testharness.load_runner(new_environment["protocol"])

    def do_test(self, test):
        url = self.test_url(test)

        success, data = WebDriverRun(self.logger,
                                     self.do_testharness,
                                     self.protocol,
                                     url,
                                     test.timeout * self.timeout_multiplier,
                                     self.extra_timeout).run()

        if success:
            return self.convert_result(test, data)

        return (test.result_cls(*data), [])

    def do_testharness(self, protocol, url, timeout):
        # The previous test may not have closed its old windows (if something
        # went wrong or if cleanup_after_test was False), so clean up here.
        parent_window = protocol.testharness.close_old_windows()

        # Now start the test harness
        protocol.testharness.open_test_window(self.window_id)
        test_window = protocol.testharness.get_test_window(self.window_id,
                                                           parent_window,
                                                           timeout=5*self.timeout_multiplier)
        self.protocol.base.set_window(test_window)

        # Wait until about:blank has been loaded
        protocol.base.execute_script(self.window_loaded_script, asynchronous=True)

        handler = WebDriverCallbackHandler(self.logger, protocol, test_window)
        protocol.webdriver.url = url

        while True:
            result = protocol.base.execute_script(
                self.script_resume, asynchronous=True, args=[strip_server(url)])

            # As of 2019-03-29, WebDriver does not define expected behavior for
            # cases where the browser crashes during script execution:
            #
            # https://github.com/w3c/webdriver/issues/1308
            if not isinstance(result, list) or len(result) != 2:
                try:
                    is_alive = self.is_alive()
                except error.WebDriverException:
                    is_alive = False

                if not is_alive:
                    raise Exception("Browser crashed during script execution.")

            done, rv = handler(result)
            if done:
                break

        # Attempt to cleanup any leftover windows, if allowed. This is
        # preferable as it will blame the correct test if something goes wrong
        # closing windows, but if the user wants to see the test results we
        # have to leave the window(s) open.
        if self.cleanup_after_test:
            protocol.testharness.close_old_windows()

        return rv


class WebDriverRefTestExecutor(RefTestExecutor):
    protocol_cls = WebDriverProtocol

    def __init__(self, logger, browser, server_config, timeout_multiplier=1,
                 screenshot_cache=None, close_after_done=True,
                 debug_info=None, capabilities=None, debug_test=False,
                 reftest_screenshot="unexpected", **kwargs):
        """WebDriver-based executor for reftests"""
        RefTestExecutor.__init__(self,
                                 logger,
                                 browser,
                                 server_config,
                                 screenshot_cache=screenshot_cache,
                                 timeout_multiplier=timeout_multiplier,
                                 debug_info=debug_info,
                                 reftest_screenshot=reftest_screenshot)
        self.protocol = self.protocol_cls(self,
                                          browser,
                                          capabilities=capabilities)
        self.implementation = RefTestImplementation(self)
        self.close_after_done = close_after_done
        self.has_window = False
        self.debug_test = debug_test

        with open(os.path.join(here, "test-wait.js")) as f:
            self.wait_script = f.read() % {"classname": "reftest-wait"}

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
        except error.InvalidArgumentException:
            # Safari 12 throws with 0 or 1, treating them as bools; fixed in STP
            self.protocol.webdriver.window.position = (2, 2)
        self.protocol.webdriver.window.size = (800 + width_offset, 600 + height_offset)

        result = self.implementation.run_test(test)

        if self.debug_test and result["status"] in ["PASS", "FAIL", "ERROR"] and "extra" in result:
            self.protocol.debug.load_reftest_analyzer(test, result)

        return self.convert_result(test, result)

    def screenshot(self, test, viewport_size, dpi, page_ranges):
        # https://github.com/web-platform-tests/wpt/issues/7135
        assert viewport_size is None
        assert dpi is None

        return WebDriverRun(self.logger,
                            self._screenshot,
                            self.protocol,
                            self.test_url(test),
                            test.timeout,
                            self.extra_timeout).run()

    def _screenshot(self, protocol, url, timeout):
        self.protocol.base.load(url)

        self.protocol.base.execute_script(self.wait_script, True)

        screenshot = self.protocol.webdriver.screenshot()
        if screenshot is None:
            raise ValueError('screenshot is None')

        # strip off the data:img/png, part of the url
        if screenshot.startswith("data:image/png;base64,"):
            screenshot = screenshot.split(",", 1)[1]

        return screenshot


class WebDriverCrashtestExecutor(CrashtestExecutor):
    protocol_cls = WebDriverProtocol

    def __init__(self, logger, browser, server_config, timeout_multiplier=1,
                 screenshot_cache=None, close_after_done=True,
                 debug_info=None, capabilities=None, **kwargs):
        """WebDriver-based executor for crashtests"""
        CrashtestExecutor.__init__(self,
                                   logger,
                                   browser,
                                   server_config,
                                   screenshot_cache=screenshot_cache,
                                   timeout_multiplier=timeout_multiplier,
                                   debug_info=debug_info)
        self.protocol = self.protocol_cls(self,
                                          browser,
                                          capabilities=capabilities)

        with open(os.path.join(here, "test-wait.js")) as f:
            self.wait_script = f.read() % {"classname": "test-wait"}

    def do_test(self, test):
        timeout = (test.timeout * self.timeout_multiplier if self.debug_info is None
                   else None)

        success, data = WebDriverRun(self.logger,
                                     self.do_crashtest,
                                     self.protocol,
                                     self.test_url(test),
                                     timeout,
                                     self.extra_timeout).run()

        if success:
            return self.convert_result(test, data)

        return (test.result_cls(*data), [])

    def do_crashtest(self, protocol, url, timeout):
        protocol.base.load(url)
        protocol.base.execute_script(self.wait_script, asynchronous=True)

        return {"status": "PASS",
                "message": None}
