# mypy: allow-untyped-defs

import asyncio
import json
import os
import socket
import threading
import time
import traceback
import uuid
from urllib.parse import urljoin

from .base import (AsyncCallbackHandler,
                   CallbackHandler,
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
                       BidiBrowsingContextProtocolPart,
                       BidiEventsProtocolPart,
                       BidiScriptProtocolPart,
                       DevicePostureProtocolPart,
                       StorageProtocolPart,
                       merge_dicts)

from typing import List, Optional, Tuple
from webdriver.client import Session
from webdriver import error as webdriver_error
from webdriver.bidi import error as webdriver_bidi_error
from webdriver.bidi.protocol import bidi_deserialize

here = os.path.dirname(__file__)


class WebDriverCallbackHandler(CallbackHandler):
    unimplemented_exc = (NotImplementedError, webdriver_error.UnknownCommandException)
    expected_exc = (webdriver_error.WebDriverException,)


class WebDriverAsyncCallbackHandler(AsyncCallbackHandler):
    unimplemented_exc = (NotImplementedError, webdriver_bidi_error.UnknownCommandException)
    expected_exc = (webdriver_bidi_error.BidiException,)


class WebDriverBaseProtocolPart(BaseProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def execute_script(self, script, asynchronous=False, args=None):
        method = self.webdriver.execute_async_script if asynchronous else self.webdriver.execute_script
        return method(script, args=args)

    def set_timeout(self, timeout):
        try:
            self.webdriver.timeouts.script = timeout
        except webdriver_error.WebDriverException:
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
            except (webdriver_error.TimeoutException,
                    webdriver_error.ScriptTimeoutException,
                    webdriver_error.JavascriptErrorException):
                # A JavascriptErrorException will happen when we navigate;
                # by ignoring it it's possible to reload the test whilst the
                # harness remains paused
                pass
            except (socket.timeout, webdriver_error.NoSuchWindowException, webdriver_error.UnknownErrorException, OSError):
                break
            except Exception:
                message = "Uncaught exception in WebDriverBaseProtocolPart.wait:\n"
                message += traceback.format_exc()
                self.logger.error(message)
                break
        return False


class WebDriverBidiBrowsingContextProtocolPart(BidiBrowsingContextProtocolPart):
    def __init__(self, parent):
        super().__init__(parent)
        self.webdriver = None

    def setup(self):
        self.webdriver = self.parent.webdriver

    async def handle_user_prompt(self,
                                 context: str,
                                 accept: Optional[bool] = None,
                                 user_text: Optional[str] = None) -> None:
        await self.webdriver.bidi_session.browsing_context.handle_user_prompt(
            context=context, accept=accept, user_text=user_text)


class WebDriverBidiEventsProtocolPart(BidiEventsProtocolPart):
    _subscriptions: List[Tuple[List[str], Optional[List[str]]]] = []

    def __init__(self, parent):
        super().__init__(parent)
        self.webdriver = None

    def setup(self):
        self.webdriver = self.parent.webdriver

    async def _contexts_to_top_contexts(self, contexts: Optional[List[str]]) -> Optional[List[str]]:
        """Gathers the list of top-level contexts for the given list of contexts."""
        if contexts is None:
            # Global subscription.
            return None
        top_contexts = set()
        for context in contexts:
            maybe_top_context = await self._get_top_context(context)
            if maybe_top_context is not None:
                # The context is found. Add its top-level context to the result set.
                top_contexts.add(maybe_top_context)
        return list(top_contexts)

    async def _get_top_context(self, context: str) -> Optional[str]:
        """Returns the top context id for the given context id."""
        # It is done in suboptimal way by calling `getTree` for each parent context until reaches the top context.
        # TODO: optimise. Construct the tree once and then traverse it.
        get_tree_result = await self.webdriver.bidi_session.browsing_context.get_tree(root=context)
        if not get_tree_result:
            # The context is not found. Nothing to do.
            return None
        assert len(get_tree_result) == 1, "The context should be unique."
        context_info = get_tree_result[0]
        if context_info["parent"] is None:
            # The context is top-level. Return its ID.
            return context
        return await self._get_top_context(context_info["parent"])

    async def subscribe(self, events, contexts):
        self.logger.info("Subscribing to events %s in %s" % (events, contexts))
        # The BiDi subscriptions are done for top context even if the sub-context is provided. We need to get the
        # top-level contexts list to handle the scenario when subscription is done for a sub-context which is closed
        # afterwards. However, the subscription itself is done for the provided contexts in order to throw in case of
        # the sub-context is removed.
        top_contexts = await self._contexts_to_top_contexts(contexts)
        result = await self.webdriver.bidi_session.session.subscribe(events=events, contexts=contexts)
        # The `subscribe` method either raises an exception or adds subscription. The command is atomic, meaning in case
        # of exception no subscription is added.
        self._subscriptions.append((events, top_contexts))
        return result

    async def unsubscribe_all(self):
        self.logger.info("Unsubscribing from all the events")
        while self._subscriptions:
            events, contexts = self._subscriptions.pop()
            self.logger.debug("Unsubscribing from events %s in %s" % (events, contexts))
            try:
                await self.webdriver.bidi_session.session.unsubscribe(events=events, contexts=contexts)
            except webdriver_bidi_error.NoSuchFrameException:
                # The browsing context is already removed. Nothing to do.
                pass
            except Exception as e:
                self.logger.error("Failed to unsubscribe from events %s in %s: %s" % (events, contexts, e))
                # Re-raise the exception to identify regressions.
                # TODO: consider to continue the loop in case of the exception.
                raise e

    def add_event_listener(self, name, fn):
        self.logger.info("adding event listener %s" % name)
        return self.webdriver.bidi_session.add_event_listener(name=name, fn=fn)


class WebDriverBidiScriptProtocolPart(BidiScriptProtocolPart):
    def __init__(self, parent):
        super().__init__(parent)
        self.webdriver = None

    def setup(self):
        self.webdriver = self.parent.webdriver

    async def call_function(self, function_declaration, target, arguments=None):
        return await self.webdriver.bidi_session.script.call_function(
            function_declaration=function_declaration,
            arguments=arguments,
            target=target,
            await_promise=True)


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
        except webdriver_error.NoSuchWindowException:
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
            except webdriver_error.JavascriptErrorException:
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
        except webdriver_error.NoSuchCookieException:
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
        except webdriver_error.UnknownErrorException as e:
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
        except (webdriver_error.StaleElementReferenceException,
                webdriver_error.NoSuchFrameException) as e:
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

class WebDriverDevicePostureProtocolPart(DevicePostureProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def set_device_posture(self, posture):
        body = {"posture": posture}
        return self.webdriver.send_session_command("POST", "deviceposture", body)

    def clear_device_posture(self):
        return self.webdriver.send_session_command("DELETE", "deviceposture")

class WebDriverStorageProtocolPart(StorageProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def run_bounce_tracking_mitigations(self):
        return self.webdriver.send_session_command("DELETE", "storage/run_bounce_tracking_mitigations")

class WebDriverProtocol(Protocol):
    enable_bidi = False
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
                  WebDriverVirtualSensorPart,
                  WebDriverDevicePostureProtocolPart,
                  WebDriverStorageProtocolPart]

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
        """Connect to browser via WebDriver and crete a WebDriver session."""
        self.logger.debug("Connecting to WebDriver on URL: %s" % self.url)

        host, port = self.url.split(":")[1].strip("/"), self.url.split(':')[-1].strip("/")

        capabilities = {"alwaysMatch": self.capabilities}
        self.webdriver = Session(host, port, capabilities=capabilities, enable_bidi=self.enable_bidi)
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

    def is_alive(self) -> bool:
        if not self.webdriver:
            return False
        try:
            # Get a simple property over the connection, with 2 seconds of timeout
            # that should be more than enough to check if the WebDriver its
            # still alive, and allows to complete the check within the testrunner
            # 5 seconds of extra_timeout we have as maximum to end the test before
            # the external timeout from testrunner triggers.
            self.webdriver.send_session_command("GET", "window", timeout=2)
        except (OSError, webdriver_error.WebDriverException, socket.timeout,
                webdriver_error.UnknownErrorException,
                webdriver_error.InvalidSessionIdException):
            return False
        return True

    def after_connect(self):
        self.testharness.load_runner(self.executor.last_environment["protocol"])


class WebDriverBidiProtocol(WebDriverProtocol):
    enable_bidi = True
    implements = [WebDriverBidiBrowsingContextProtocolPart,
                  WebDriverBidiEventsProtocolPart,
                  WebDriverBidiScriptProtocolPart,
                  *(part for part in WebDriverProtocol.implements)
                  ]

    def __init__(self, executor, browser, capabilities, **kwargs):
        super().__init__(executor, browser, capabilities, **kwargs)
        self.loop = asyncio.new_event_loop()

    def connect(self):
        super().connect()
        self.loop.run_until_complete(self.webdriver.bidi_session.start(self.loop))

    def teardown(self):
        try:
            self.loop.run_until_complete(self.webdriver.bidi_session.end())
        except Exception as e:
            message = str(getattr(e, "message", ""))
            if message:
                message += "\n"
            message += traceback.format_exc()
            self.logger.debug(message)
        self.loop.stop()
        super().teardown()


class WebDriverRun(TimedRunner):
    def set_timeout(self):
        try:
            self.protocol.base.set_timeout(self.timeout + self.extra_timeout)
        except webdriver_error.UnknownErrorException:
            msg = "Lost WebDriver connection"
            self.logger.error(msg)
            return ("INTERNAL-ERROR", msg)

    def run_func(self):
        try:
            self.result = True, self.func(self.protocol, self.url, self.timeout)
        except (webdriver_error.TimeoutException, webdriver_error.ScriptTimeoutException):
            self.result = False, ("EXTERNAL-TIMEOUT", None)
        except socket.timeout:
            # Checking if the browser is alive below is likely to hang, so mark
            # this case as a CRASH unconditionally.
            self.result = False, ("CRASH", None)
        except Exception as e:
            if (isinstance(e, webdriver_error.WebDriverException) and
                    e.http_status == 408 and
                    e.status_code == "asynchronous script timeout"):
                # workaround for https://bugs.chromium.org/p/chromedriver/issues/detail?id=2001
                self.result = False, ("EXTERNAL-TIMEOUT", None)
            else:
                status = "INTERNAL-ERROR" if self.protocol.is_alive() else "CRASH"
                message = str(getattr(e, "message", ""))
                if message:
                    message += "\n"
                message += traceback.format_exc()
                self.result = False, (status, message)
        finally:
            self.result_flag.set()


class WebDriverTestharnessExecutor(TestharnessExecutor):
    supports_testdriver = True
    protocol_cls = WebDriverProtocol
    _get_next_message = None

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

        if hasattr(self.protocol, 'bidi_script'):
            # If `bidi_script` is available, the messages can be handled via BiDi.
            self._get_next_message = self._get_next_message_bidi
        else:
            self._get_next_message = self._get_next_message_classic

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

        return (test.make_result(*data), [])

    def do_testharness(self, protocol, url, timeout):
        # The previous test may not have closed its old windows (if something
        # went wrong or if cleanup_after_test was False), so clean up here.
        parent_window = protocol.testharness.close_old_windows()

        # If protocol implements `bidi_events`, remove all the existing subscriptions.
        if hasattr(protocol, 'bidi_events'):
            # Use protocol loop to run the async cleanup.
            protocol.loop.run_until_complete(protocol.bidi_events.unsubscribe_all())

        # Now start the test harness
        protocol.testharness.open_test_window(self.window_id)
        test_window = protocol.testharness.get_test_window(self.window_id,
                                                           parent_window,
                                                           timeout=5*self.timeout_multiplier)
        self.protocol.base.set_window(test_window)

        # Wait until about:blank has been loaded
        protocol.base.execute_script(self.window_loaded_script, asynchronous=True)

        # Exceptions occurred outside the main loop.
        unexpected_exceptions = []

        if hasattr(protocol, 'bidi_events'):
            # If protocol implements `bidi_events`, forward all the events to test_driver.
            async def process_bidi_event(method, params):
                try:
                    self.logger.debug(f"Received bidi event: {method}, {params}")
                    if hasattr(protocol, 'bidi_browsing_context') and method == "browsingContext.userPromptOpened" and \
                            params["context"] == test_window:
                        # User prompts of the test window are handled separately. In classic implementation, this user
                        # prompt always causes an exception when `_get_next_message` is called. In BiDi it's not a case,
                        # as the BiDi protocol allows sending commands even with the user prompt opened. However, the
                        # user prompt can block the testdriver JS execution and cause the dead loop. To overcome this
                        # issue, the user prompt of the test window is always dismissed and the test is failing.
                        try:
                            await protocol.bidi_browsing_context.handle_user_prompt(params["context"])
                        except Exception as e:
                            if "no such alert" in str(e):
                                # The user prompt is already dismissed by WebDriver BiDi server. Ignore the exception.
                                pass
                            else:
                                # The exception is unexpected. Re-raising it to handle it in the main loop.
                                raise e
                        raise Exception("Unexpected user prompt in test window: %s" % params)
                    else:
                        protocol.testdriver.send_message(-1, "event", method, json.dumps({
                            "params": params,
                            "method": method}))
                except Exception as e:
                    # As the event listener is async, the exceptions should be added to the list to be processed in the
                    # main loop.
                    self.logger.error("BiDi event processing failed: %s" % e)
                    unexpected_exceptions.append(e)

            protocol.bidi_events.add_event_listener(None, process_bidi_event)
            protocol.loop.run_until_complete(protocol.bidi_events.subscribe(['browsingContext.userPromptOpened'], None))

        # If possible, support async actions.
        if hasattr(protocol, 'loop'):
            handler = WebDriverAsyncCallbackHandler(self.logger, protocol, test_window, protocol.loop)
        else:
            handler = WebDriverCallbackHandler(self.logger, protocol, test_window)

        protocol.webdriver.url = url

        while True:
            if len(unexpected_exceptions) > 0:
                # TODO: what to do if there are more then 1 unexpected exceptions?
                raise unexpected_exceptions[0]

            test_driver_message = self._get_next_message(protocol, url, test_window)
            self.logger.debug("Receive message from testdriver: %s" % test_driver_message)

            # As of 2019-03-29, WebDriver does not define expected behavior for
            # cases where the browser crashes during script execution:
            #
            # https://github.com/w3c/webdriver/issues/1308
            if not isinstance(test_driver_message, list) or len(test_driver_message) != 3:
                try:
                    is_alive = self.is_alive()
                except webdriver_error.WebDriverException:
                    is_alive = False
                if not is_alive:
                    raise Exception("Browser crashed during script execution.")

            # In case of WebDriver Classic, a user prompt created after starting execution of the resume script will
            # resolve the script with `null` [1, 2]. In that case, cycle this event loop and handle the prompt the next
            # time the resume script executes.
            #
            # [1]: Step 5.3 of https://www.w3.org/TR/webdriver/#execute-async-script
            # [2]: https://www.w3.org/TR/webdriver/#dfn-execute-a-function-body
            if test_driver_message is None:
                continue

            done, rv = handler(test_driver_message)
            if done:
                break

        # If protocol implements `bidi_events`, remove all the existing subscriptions.
        if hasattr(protocol, 'bidi_events'):
            # Use protocol loop to run the async cleanup.
            protocol.loop.run_until_complete(protocol.bidi_events.unsubscribe_all())

        # Attempt to clean up any leftover windows, if allowed. This is
        # preferable as it will blame the correct test if something goes wrong
        # closing windows, but if the user wants to see the test results we
        # have to leave the window(s) open.
        if self.cleanup_after_test:
            protocol.testharness.close_old_windows()

        if len(unexpected_exceptions) > 0:
            # TODO: what to do if there are more then 1 unexpected exceptions?
            raise unexpected_exceptions[0]

        return rv

    def _get_next_message_classic(self, protocol, url, _):
        """
        Get the next message from the test_driver using the classic WebDriver async script execution. This will block
        the event loop until the test_driver send a message.
        :param window:
        """
        return protocol.base.execute_script(self.script_resume, asynchronous=True, args=[strip_server(url)])

    def _get_next_message_bidi(self, protocol, url, test_window):
        """
        Get the next message from the test_driver using async call. This will not block the event loop, which allows for
        processing the events from the test_runner to test_driver while waiting for the next test_driver commands.
        """
        # As long as we want to be able to use scripts both in bidi and in classic mode, the script should
        # be wrapped to some harness to emulate the WebDriver Classic async script execution. The script
        # will be provided with the `resolve` delegate, which finishes the execution. After that the
        # coroutine is finished as well.
        wrapped_script = """async function(...args){
                        return new Promise((resolve, reject) => {
                            args.push(resolve);
                            (async function(){
                                %s
                            }).apply(null, args);
                        })
                    }""" % self.script_resume

        bidi_url_argument = {
            "type": "string",
            "value": strip_server(url)
        }

        # `run_until_complete` allows processing BiDi events in the same loop while waiting for the next message.
        message = protocol.loop.run_until_complete(protocol.bidi_script.call_function(
            wrapped_script, target={
                "context": test_window
            },
            arguments=[bidi_url_argument]))
        # The message is in WebDriver BiDi format. Deserialize it.
        deserialized_message = bidi_deserialize(message)
        return deserialized_message


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
        # width_offset and height_offset should never be negative
        width_offset = max(width_offset, 0)
        height_offset = max(height_offset, 0)
        try:
            self.protocol.webdriver.window.position = (0, 0)
        except webdriver_error.InvalidArgumentException:
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

        return (test.make_result(*data), [])

    def do_crashtest(self, protocol, url, timeout):
        protocol.base.load(url)
        protocol.base.execute_script(self.wait_script, asynchronous=True)

        return {"status": "PASS",
                "message": None}
