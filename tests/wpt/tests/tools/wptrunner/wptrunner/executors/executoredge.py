# mypy: allow-untyped-defs

import os

from .executorwebdriver import (
    WebDriverCrashtestExecutor,
    WebDriverRefTestExecutor,
    WebDriverRun,
    WebDriverTestharnessExecutor,
)

from .executorchrome import (
    ChromeDriverProtocol,
    make_sanitizer_mixin,
)

here = os.path.dirname(__file__)

_SanitizerMixin = make_sanitizer_mixin(WebDriverCrashtestExecutor)


class EdgeDriverProtocol(ChromeDriverProtocol):
    vendor_prefix = "ms"


class EdgeDriverRefTestExecutor(WebDriverRefTestExecutor, _SanitizerMixin):  # type: ignore
    protocol_cls = EdgeDriverProtocol


class EdgeDriverTestharnessExecutor(WebDriverTestharnessExecutor, _SanitizerMixin):  # type: ignore
    protocol_cls = EdgeDriverProtocol

    def __init__(self, *args, reuse_window=False, **kwargs):
        super().__init__(*args, **kwargs)
        self.protocol.reuse_window = reuse_window


class EdgeDriverPrintRefTestExecutor(EdgeDriverRefTestExecutor):
    protocol_cls = EdgeDriverProtocol

    def setup(self, runner, protocol=None):
        super().setup(runner, protocol)
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
