# mypy: allow-untyped-defs

import collections
import json
import os
import re
import time
import uuid
from typing import Mapping, MutableMapping

from webdriver import error

from .base import strip_server
from .executorwebdriver import (
    WebDriverBaseProtocolPart,
    WebDriverCrashtestExecutor,
    WebDriverFedCMProtocolPart,
    WebDriverPrintRefTestExecutor,
    WebDriverProtocol,
    WebDriverBidiProtocol,
    WebDriverRefTestExecutor,
    WebDriverTestDriverProtocolPart,
    WebDriverTestharnessExecutor,
    WebDriverTestharnessProtocolPart,
)
from .protocol import LeakProtocolPart, ProtocolPart

here = os.path.dirname(__file__)


class ChromeDriverBaseProtocolPart(WebDriverBaseProtocolPart):
    def create_window(self, type="tab", **kwargs):
        try:
            return super().create_window(type=type, **kwargs)
        except error.WebDriverException:
            # TODO(crbug.com/375275185): This case exists solely as a workaround
            # for Android WebView not supporting "new window". Once fixed, just
            # use the standard `WebDriverBaseProtocolPart`.
            window_id = str(uuid.uuid4())
            self.webdriver.execute_script(
                "window.open('about:blank', '%s', 'noopener')" % window_id)
            return self._get_test_window(window_id, self.current_window)

    def _get_test_window(self, window_id, parent, timeout=5):
        """Find the test window amongst all the open windows.
        This is assumed to be either the named window or the one after the parent in the list of
        window handles
        :param window_id: The DOM name of the Window
        :param parent: The handle of the current window
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


class ChromeDriverLeakProtocolPart(LeakProtocolPart):
    def get_counters(self) -> Mapping[str, int]:
        response = self.parent.cdp.execute_cdp_command("Memory.getDOMCountersForLeakDetection")
        counters: MutableMapping[str, int] = collections.Counter({
            counter["name"]: counter["count"]
            for counter in response["counters"]
        })
        # Exclude resources associated with User Agent CSS from leak detection,
        # since they are persisted through page navigation.
        counters["live_resources"] -= counters.pop("live_ua_css_resources", 0)
        return counters


class ChromeDriverTestDriverProtocolPart(WebDriverTestDriverProtocolPart):
    """An interface to the browser-side testdriver infrastructure that lazily settles calls."""

    def setup(self):
        super().setup()
        self._pending_message = ""

    def send_message(self, cmd_id, message_type, status, message=None):
        message_script = self._format_send_message_script(cmd_id, message_type, status, message)
        if message_type == "complete":
            assert not self._pending_message, self._pending_message
            self._pending_message = message_script
        else:
            self.webdriver.execute_script(message_script)

    def _get_next_message_classic(self, url, script_resume):
        try:
            message_script, self._pending_message = self._pending_message, ""
            return self.parent.base.execute_script(message_script + script_resume,
                                                   asynchronous=True,
                                                   args=[strip_server(url)])
        except error.JavascriptErrorException as js_error:
            # TODO(crbug.com/340662810): Cycle testdriver event loop to work
            # around `testharnessreport.js` flakily not loaded.
            if re.search(r'window\.__wptrunner_process_next_event is not a function',
                         js_error.message):
                time.sleep(0.05)
                return None
            raise


class ChromeDriverTestharnessProtocolPart(WebDriverTestharnessProtocolPart):
    """Implementation of `testharness.js` tests controlled by ChromeDriver.

    The main difference from the default WebDriver testharness implementation is
    that the test window can be reused between tests for better performance.
    """
    def reset_browser_state(self):
        for command, params in [
            # Reset default permissions that `test_driver.set_permission(...)`
            # may have altered.
            ("Browser.resetPermissions", None),
            # Chromium requires the `background-sync` permission for reporting
            # APIs to work. Not all embedders (notably, `chrome --headless=old`)
            # grant `background-sync` by default, so this CDP call ensures the
            # permission is granted for all origins, in line with the background
            # sync spec's recommendation [0].
            #
            # WebDriver's "Set Permission" command can only act on the test's
            # origin, which may be too limited.
            #
            # [0]: https://wicg.github.io/background-sync/spec/#permission
            ("Browser.setPermission", {
                "permission": {"name": "background-sync"},
                "setting": "granted",
            }),
        ]:
            try:
                self.parent.cdp.execute_cdp_command(command, params)
            except error.WebDriverException:
                pass


class ChromeDriverFedCMProtocolPart(WebDriverFedCMProtocolPart):
    def confirm_idp_login(self):
        return self.webdriver.send_session_command("POST",
                                                   f"{self.parent.vendor_prefix}/fedcm/confirmidplogin")


class ChromeDriverDevToolsProtocolPart(ProtocolPart):
    """A low-level API for sending Chrome DevTools Protocol [0] commands directly to the browser.

    Prefer using standard APIs where possible.

    [0]: https://chromedevtools.github.io/devtools-protocol/
    """
    name = "cdp"

    def setup(self):
        self.webdriver = self.parent.webdriver

    def execute_cdp_command(self, command, params=None):
        body = {"cmd": command, "params": params or {}}
        return self.webdriver.send_session_command("POST",
                                                   f"{self.parent.vendor_prefix}/cdp/execute",
                                                   body=body)


class ChromeDriverProtocol(WebDriverProtocol):
    implements = [
        ChromeDriverBaseProtocolPart,
        ChromeDriverDevToolsProtocolPart,
        ChromeDriverFedCMProtocolPart,
        ChromeDriverTestDriverProtocolPart,
        ChromeDriverTestharnessProtocolPart,
    ]
    for base_part in WebDriverProtocol.implements:
        if base_part.name not in {part.name for part in implements}:
            implements.append(base_part)

    # Prefix to apply to vendor-specific WebDriver extension commands.
    vendor_prefix = "goog"

    def __init__(self, executor, browser, capabilities, **kwargs):
        self.implements = list(ChromeDriverProtocol.implements)
        if getattr(browser, "leak_check", False):
            self.implements.append(ChromeDriverLeakProtocolPart)
        super().__init__(executor, browser, capabilities, **kwargs)


class ChromeDriverBidiProtocol(WebDriverBidiProtocol):
    implements = [
        ChromeDriverBaseProtocolPart,
        ChromeDriverDevToolsProtocolPart,
        ChromeDriverFedCMProtocolPart,
        ChromeDriverTestharnessProtocolPart,
    ]
    for base_part in WebDriverBidiProtocol.implements:
        if base_part.name not in {part.name for part in implements}:
            implements.append(base_part)

    # Prefix to apply to vendor-specific WebDriver extension commands.
    vendor_prefix = "goog"

    def __init__(self, executor, browser, capabilities, **kwargs):
        self.implements = list(ChromeDriverBidiProtocol.implements)
        if getattr(browser, "leak_check", False):
            self.implements.append(ChromeDriverLeakProtocolPart)
        super().__init__(executor, browser, capabilities, **kwargs)


def _evaluate_sanitized_result(executor_cls):
    if hasattr(executor_cls, "base_convert_result"):
        # Don't wrap more than once, which can cause unbounded recursion.
        return executor_cls
    executor_cls.base_convert_result = executor_cls.convert_result

    def convert_result(self, test, result, **kwargs):
        test_result, subtest_results = self.base_convert_result(test, result, **kwargs)
        if test_result.extra.get("leak_counters"):
            test_result = test.make_result("CRASH",
                                           test_result.message,
                                           test_result.expected,
                                           test_result.extra,
                                           test_result.stack,
                                           test_result.known_intermittent)
        if self.sanitizer_enabled:
            # Coerce functional failures to OK/PASS, and discard any subtest results.
            if test_result.status in {"ERROR", "FAIL", "INTERNAL-ERROR", "PRECONDITION_FAILED"}:
                test_result.status = test_result.default_expected
            return test_result, []
        return test_result, subtest_results

    executor_cls.convert_result = convert_result
    return executor_cls


@_evaluate_sanitized_result
class ChromeDriverCrashTestExecutor(WebDriverCrashtestExecutor):
    protocol_cls = ChromeDriverProtocol

    def __init__(self, *args, sanitizer_enabled=False, **kwargs):
        super().__init__(*args, **kwargs)
        self.sanitizer_enabled = sanitizer_enabled


@_evaluate_sanitized_result
class ChromeDriverRefTestExecutor(WebDriverRefTestExecutor):
    protocol_cls = ChromeDriverProtocol

    def __init__(self, *args, sanitizer_enabled=False, **kwargs):
        super().__init__(*args, **kwargs)
        self.sanitizer_enabled = sanitizer_enabled


@_evaluate_sanitized_result
class ChromeDriverTestharnessExecutor(WebDriverTestharnessExecutor):

    def __init__(self, *args, sanitizer_enabled=False, reuse_window=False, **kwargs):
        require_webdriver_bidi = kwargs.get("browser_settings", {}).get(
            "require_webdriver_bidi", None)
        if require_webdriver_bidi:
            self.protocol_cls = ChromeDriverBidiProtocol
        else:
            self.protocol_cls = ChromeDriverProtocol

        super().__init__(*args, **kwargs)
        self.sanitizer_enabled = sanitizer_enabled
        self.reuse_window = reuse_window

    def get_or_create_test_window(self, protocol):
        test_window = self.protocol.testharness.persistent_test_window
        if test_window:
            try:
                # Mimic the "new window" WebDriver command by loading `about:blank`
                # with no other browsing history.
                protocol.base.set_window(test_window)
                protocol.base.load("about:blank")
                # DevTools can very rarely fail with "History cannot be pruned".
                # The test window will be replaced in that case.
                protocol.cdp.execute_cdp_command("Page.resetNavigationHistory")
            except error.WebDriverException:
                protocol.testharness.close_windows([test_window])
                protocol.base.set_window(protocol.testharness.runner_handle)
                test_window = self.protocol.testharness.persistent_test_window = None
        if not test_window:
            test_window = super().get_or_create_test_window(protocol)
        if self.reuse_window:
            self.protocol.testharness.persistent_test_window = test_window
        return test_window


@_evaluate_sanitized_result
class ChromeDriverPrintRefTestExecutor(WebDriverPrintRefTestExecutor):
    protocol_cls = ChromeDriverProtocol

    def __init__(self, *args, sanitizer_enabled=False, **kwargs):
        super().__init__(*args, **kwargs)
        self.sanitizer_enabled = sanitizer_enabled
