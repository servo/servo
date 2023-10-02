# mypy: allow-untyped-defs

import os
import time
import traceback
from typing import Type
from urllib.parse import urljoin

from webdriver import error

from .base import (
    CrashtestExecutor,
    TestharnessExecutor,
    get_pages,
)
from .executorwebdriver import (
    WebDriverCrashtestExecutor,
    WebDriverFedCMProtocolPart,
    WebDriverProtocol,
    WebDriverRefTestExecutor,
    WebDriverRun,
    WebDriverTestharnessExecutor,
    WebDriverTestharnessProtocolPart,
)
from .protocol import PrintProtocolPart

here = os.path.dirname(__file__)


def make_sanitizer_mixin(crashtest_executor_cls: Type[CrashtestExecutor]):  # type: ignore[no-untyped-def]
    class SanitizerMixin:
        def __new__(cls, logger, browser, **kwargs):
            # Overriding `__new__` is the least worst way we can force tests to run
            # as crashtests at runtime while still supporting:
            #   * Class attributes (e.g., `extra_timeout`)
            #   * Pickleability for `multiprocessing` transport
            #   * The `__wptrunner__` product interface
            #
            # These requirements rule out approaches with `functools.partial(...)`
            # or global variables.
            if kwargs.get("sanitizer_enabled"):
                executor = crashtest_executor_cls(logger, browser, **kwargs)

                def convert_from_crashtest_result(test, result):
                    if issubclass(cls, TestharnessExecutor):
                        status = result["status"]
                        if status == "PASS":
                            status = "OK"
                        harness_result = test.result_cls(status, result["message"])
                        # Don't report subtests.
                        return harness_result, []
                    # `crashtest` statuses are a subset of `(print-)reftest`
                    # ones, so no extra conversion necessary.
                    return cls.convert_result(executor, test, result)

                executor.convert_result = convert_from_crashtest_result
                return executor
            return super().__new__(cls)
    return SanitizerMixin


_SanitizerMixin = make_sanitizer_mixin(WebDriverCrashtestExecutor)


class ChromeDriverTestharnessProtocolPart(WebDriverTestharnessProtocolPart):
    """Implementation of `testharness.js` tests controlled by ChromeDriver.

    The main difference from the default WebDriver testharness implementation is
    that the test window can be reused between tests for better performance.
    """

    def setup(self):
        super().setup()
        # Handle (an alphanumeric string) that may be set if window reuse is
        # enabled. This state allows the protocol to distinguish the test
        # window from other windows a test itself may create that the "Get
        # Window Handles" command also returns.
        #
        # Because test window persistence is a Chrome-only feature, it's not
        # exposed to the base WebDriver testharness executor.
        self.test_window = None
        self.reuse_window = self.parent.reuse_window
        # Company prefix to apply to vendor-specific WebDriver extension commands.
        self.cdp_company_prefix = "goog"

    def close_test_window(self):
        if self.test_window:
            self._close_window(self.test_window)
            self.test_window = None

    def close_old_windows(self):
        self.webdriver.actions.release()
        for handle in self.webdriver.handles:
            if handle not in {self.runner_handle, self.test_window}:
                self._close_window(handle)
        if not self.reuse_window:
            self.close_test_window()
        self.webdriver.window_handle = self.runner_handle
        return self.runner_handle

    def open_test_window(self, window_id):
        if self.test_window:
            # Try to reuse the existing test window by emulating the `about:blank`
            # page with no history you would get with a new window.
            try:
                self.webdriver.window_handle = self.test_window
                # Reset navigation history with Chrome DevTools Protocol:
                # https://chromedevtools.github.io/devtools-protocol/tot/Page/#method-resetNavigationHistory
                body = {
                    "cmd": "Page.resetNavigationHistory",
                    "params": {},
                }
                self.webdriver.send_session_command("POST",
                                                    self.cdp_company_prefix + "/cdp/execute",
                                                    body=body)
                self.webdriver.url = "about:blank"
                return
            except error.NoSuchWindowException:
                self.test_window = None
        super().open_test_window(window_id)

    def get_test_window(self, window_id, parent, timeout=5):
        if self.test_window:
            return self.test_window
        # Poll the handles endpoint for the test window like the base WebDriver
        # protocol part, but don't bother checking for the serialized
        # WindowProxy (not supported by Chrome currently).
        deadline = time.time() + timeout
        while time.time() < deadline:
            self.test_window = self._poll_handles_for_test_window(parent)
            if self.test_window is not None:
                assert self.test_window != parent
                return self.test_window
            time.sleep(0.03)
        raise Exception("unable to find test window")


class ChromeDriverPrintProtocolPart(PrintProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver
        self.runner_handle = None
        # Company prefix to apply to vendor-specific WebDriver extension commands.
        self.cdp_company_prefix = "goog"

    def load_runner(self):
        url = urljoin(self.parent.executor.server_url("http"), "/print_pdf_runner.html")
        self.logger.debug("Loading %s" % url)
        try:
            self.webdriver.url = url
        except Exception as e:
            self.logger.critical(
                "Loading initial page %s failed. Ensure that the "
                "there are no other programs bound to this port and "
                "that your firewall rules or network setup does not "
                "prevent access.\n%s" % (url, traceback.format_exc(e)))
            raise
        self.runner_handle = self.webdriver.window_handle

    def render_as_pdf(self, width, height):
        margin = 0.5
        body = {
            "cmd": "Page.printToPDF",
            "params": {
                # Chrome accepts dimensions in inches; we are using cm
                "paperWidth": width / 2.54,
                "paperHeight": height / 2.54,
                "marginLeft": margin,
                "marginRight": margin,
                "marginTop": margin,
                "marginBottom": margin,
                "shrinkToFit": False,
                "printBackground": True,
            }
        }
        return self.webdriver.send_session_command("POST",
                                                   self.cdp_company_prefix + "/cdp/execute",
                                                   body=body)["data"]

    def pdf_to_png(self, pdf_base64, ranges):
        handle = self.webdriver.window_handle
        self.webdriver.window_handle = self.runner_handle
        try:
            rv = self.webdriver.execute_async_script("""
let callback = arguments[arguments.length - 1];
render('%s').then(result => callback(result))""" % pdf_base64)
            page_numbers = get_pages(ranges, len(rv))
            rv = [item for i, item in enumerate(rv) if i + 1 in page_numbers]
            return rv
        finally:
            self.webdriver.window_handle = handle


class ChromeDriverFedCMProtocolPart(WebDriverFedCMProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver
        # Company prefix to apply to vendor-specific WebDriver extension commands.
        self.fedcm_company_prefix = "goog"


    def confirm_idp_login(self):
        return self.webdriver.send_session_command("POST",
                                                   self.fedcm_company_prefix + "/fedcm/confirmidplogin")


class ChromeDriverProtocol(WebDriverProtocol):
    implements = [
        ChromeDriverFedCMProtocolPart,
        ChromeDriverPrintProtocolPart,
        ChromeDriverTestharnessProtocolPart,
        *(part for part in WebDriverProtocol.implements
          if part.name != ChromeDriverTestharnessProtocolPart.name and
            part.name != ChromeDriverFedCMProtocolPart.name)
    ]
    reuse_window = False


class ChromeDriverRefTestExecutor(WebDriverRefTestExecutor, _SanitizerMixin):  # type: ignore
    protocol_cls = ChromeDriverProtocol


class ChromeDriverTestharnessExecutor(WebDriverTestharnessExecutor, _SanitizerMixin):  # type: ignore
    protocol_cls = ChromeDriverProtocol

    def __init__(self, *args, reuse_window=False, **kwargs):
        super().__init__(*args, **kwargs)
        self.protocol.reuse_window = reuse_window


class ChromeDriverPrintRefTestExecutor(ChromeDriverRefTestExecutor):
    protocol_cls = ChromeDriverProtocol

    def setup(self, runner):
        super().setup(runner)
        self.protocol.pdf_print.load_runner()
        self.has_window = False
        with open(os.path.join(here, "reftest.js")) as f:
            self.script = f.read()

    def screenshot(self, test, viewport_size, dpi, page_ranges):
        # https://github.com/web-platform-tests/wpt/issues/7140
        assert dpi is None

        if not self.has_window:
            self.protocol.base.execute_script(self.script)
            self.protocol.base.set_window(self.protocol.webdriver.handles[-1])
            self.has_window = True

        self.viewport_size = viewport_size
        self.page_ranges = page_ranges.get(test.url)
        timeout = self.timeout_multiplier * test.timeout if self.debug_info is None else None

        test_url = self.test_url(test)

        return WebDriverRun(self.logger,
                            self._render,
                            self.protocol,
                            test_url,
                            timeout,
                            self.extra_timeout).run()

    def _render(self, protocol, url, timeout):
        protocol.webdriver.url = url

        protocol.base.execute_script(self.wait_script, asynchronous=True)

        pdf = protocol.pdf_print.render_as_pdf(*self.viewport_size)
        screenshots = protocol.pdf_print.pdf_to_png(pdf, self.page_ranges)
        for i, screenshot in enumerate(screenshots):
            # strip off the data:img/png, part of the url
            if screenshot.startswith("data:image/png;base64,"):
                screenshots[i] = screenshot.split(",", 1)[1]

        return screenshots
