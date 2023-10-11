# mypy: allow-untyped-defs

from .base import RefTestExecutor, RefTestImplementation, CrashtestExecutor, TestharnessExecutor
from .executorchrome import make_sanitizer_mixin
from .protocol import Protocol, ProtocolPart
from time import time
from queue import Empty
from base64 import b64encode
import json


class CrashError(BaseException):
    pass

class LeakError(BaseException):
    pass

def _read_line(io_queue, deadline=None, encoding=None, errors="strict", raise_crash_leak=True):
    """Reads a single line from the io queue. The read must succeed before `deadline` or
    a TimeoutError is raised. The line is returned as a bytestring or optionally with the
    specified `encoding`. If `raise_crash_leak` is set, a CrashError is raised if the line
    happens to be a crash message, or a LeakError is raised if the line happens to be a
    leak message.
    """
    current_time = time()

    if deadline and current_time > deadline:
        raise TimeoutError()

    try:
        line = io_queue.get(True, deadline - current_time if deadline else None)
        if raise_crash_leak and line.startswith(b"#CRASHED"):
            raise CrashError()
        if raise_crash_leak and line.startswith(b"#LEAK"):
            raise LeakError()
    except Empty as e:
        raise TimeoutError() from e

    return line.decode(encoding, errors) if encoding else line


class ContentShellTestPart(ProtocolPart):
    """This protocol part is responsible for running tests via content_shell's protocol mode.

    For more details, see:
    https://chromium.googlesource.com/chromium/src.git/+/HEAD/content/web_test/browser/test_info_extractor.h
    """
    name = "content_shell_test"
    eof_marker = '#EOF\n'  # Marker sent by content_shell after blocks.

    def __init__(self, parent):
        super().__init__(parent)
        self.stdout_queue = parent.browser.stdout_queue
        self.stdin_queue = parent.browser.stdin_queue

    def do_test(self, command, timeout=None):
        """Send a command to content_shell and return the resulting outputs.

        A command consists of a URL to navigate to, followed by an optional
        expected image hash and 'print' mode specifier. The syntax looks like:
            http://web-platform.test:8000/test.html['<hash>['print]]
        """
        self._send_command(command)

        deadline = time() + timeout if timeout else None
        # The first block can also contain audio data but not in WPT.
        text = self._read_block(deadline)
        image = self._read_block(deadline)

        return text, image

    def _send_command(self, command):
        """Sends a single `command`, i.e. a URL to open, to content_shell.
        """
        self.stdin_queue.put((command + "\n").encode("utf-8"))

    def _read_block(self, deadline=None):
        """Tries to read a single block of content from stdout before the `deadline`.
        """
        while True:
            line = _read_line(self.stdout_queue, deadline, "latin-1").rstrip()

            if line == "Content-Type: text/plain":
                return self._read_text_block(deadline)

            if line == "Content-Type: image/png":
                return self._read_image_block(deadline)

            if line == "#EOF":
                return None

    def _read_text_block(self, deadline=None):
        """Tries to read a plain text block in utf-8 encoding before the `deadline`.
        """
        result = ""

        while True:
            line = _read_line(self.stdout_queue, deadline, "utf-8", "replace", False)

            if line.endswith(self.eof_marker):
                result += line[:-len(self.eof_marker)]
                break
            elif line.endswith('#EOF\r\n'):
                result += line[:-len('#EOF\r\n')]
                self.logger.warning('Got a CRLF-terminated #EOF - this is a driver bug.')
                break

            result += line

        return result

    def _read_image_block(self, deadline=None):
        """Tries to read an image block (as a binary png) before the `deadline`.
        """
        content_length_line = _read_line(self.stdout_queue, deadline, "utf-8")
        assert content_length_line.startswith("Content-Length:")
        content_length = int(content_length_line[15:])

        result = bytearray()

        while True:
            line = _read_line(self.stdout_queue, deadline, raise_crash_leak=False)
            excess = len(line) + len(result) - content_length

            if excess > 0:
                # This is the line that contains the EOF marker.
                assert excess == len(self.eof_marker)
                result += line[:-excess]
                break

            result += line

        return result


class ContentShellErrorsPart(ProtocolPart):
    """This protocol part is responsible for collecting the errors reported by content_shell.
    """
    name = "content_shell_errors"

    def __init__(self, parent):
        super().__init__(parent)
        self.stderr_queue = parent.browser.stderr_queue

    def read_errors(self):
        """Reads the entire content of the stderr queue as is available right now (no blocking).
        """
        result = ""

        while not self.stderr_queue.empty():
            # There is no potential for race conditions here because this is the only place
            # where we read from the stderr queue.
            result += _read_line(self.stderr_queue, None, "utf-8", "replace", False)

        return result


class ContentShellBasePart(ProtocolPart):
    """This protocol part provides functionality common to all executors.

    In particular, this protocol part implements `wait()`, which, when
    `--pause-after-test` is enabled, test runners block on until the next test
    should run.
    """
    name = "base"

    def __init__(self, parent):
        super().__init__(parent)
        self.io_stopped = parent.browser.io_stopped

    def wait(self):
        # This worker is unpaused when the browser window is closed, which this
        # `multiprocessing.Event` signals.
        self.io_stopped.wait()
        # Never rerun the test.
        return False


class ContentShellProtocol(Protocol):
    implements = [
        ContentShellBasePart,
        ContentShellTestPart,
        ContentShellErrorsPart,
    ]
    init_timeout = 10  # Timeout (seconds) to wait for #READY message.

    def connect(self):
        """Waits for content_shell to emit its "#READY" message which signals that it is fully
        initialized. We wait for a maximum of self.init_timeout seconds.
        """
        deadline = time() + self.init_timeout

        while True:
            if _read_line(self.browser.stdout_queue, deadline).rstrip() == b"#READY":
                break

    def after_connect(self):
        pass

    def teardown(self):
        # Close the queue properly to avoid broken pipe spam in the log.
        self.browser.stdin_queue.close()
        self.browser.stdin_queue.join_thread()

    def is_alive(self):
        """Checks if content_shell is alive by determining if the IO pipes are still
        open. This does not guarantee that the process is responsive.
        """
        return self.browser.io_stopped.is_set()


def _convert_exception(test, exception, errors):
    """Converts our TimeoutError and CrashError exceptions into test results.
    """
    if isinstance(exception, TimeoutError):
        return (test.result_cls("EXTERNAL-TIMEOUT", errors), [])
    if isinstance(exception, CrashError):
        return (test.result_cls("CRASH", errors), [])
    if isinstance(exception, LeakError):
        # TODO: the internal error is to force a restart, but it doesn't correctly
        # describe what the issue is. Need to find a way to return a "FAIL",
        # and restart the content_shell after the test run.
        return (test.result_cls("INTERNAL-ERROR", errors), [])
    raise exception


def timeout_for_test(executor, test):
    if executor.debug_info and executor.debug_info.interactive:
        return None
    return test.timeout * executor.timeout_multiplier


class ContentShellCrashtestExecutor(CrashtestExecutor):
    def __init__(self, logger, browser, server_config, timeout_multiplier=1, debug_info=None,
            **kwargs):
        super().__init__(logger, browser, server_config, timeout_multiplier, debug_info, **kwargs)
        self.protocol = ContentShellProtocol(self, browser)

    def do_test(self, test):
        try:
            _ = self.protocol.content_shell_test.do_test(self.test_url(test),
                                                         timeout_for_test(self, test))
            self.protocol.content_shell_errors.read_errors()
            return self.convert_result(test, {"status": "PASS", "message": None})
        except BaseException as exception:
            return _convert_exception(test, exception, self.protocol.content_shell_errors.read_errors())


_SanitizerMixin = make_sanitizer_mixin(ContentShellCrashtestExecutor)


class ContentShellRefTestExecutor(RefTestExecutor, _SanitizerMixin):  # type: ignore
    def __init__(self, logger, browser, server_config, timeout_multiplier=1, screenshot_cache=None,
            debug_info=None, reftest_screenshot="unexpected", **kwargs):
        super().__init__(logger, browser, server_config, timeout_multiplier, screenshot_cache,
                debug_info, reftest_screenshot, **kwargs)
        self.implementation = RefTestImplementation(self)
        self.protocol = ContentShellProtocol(self, browser)

    def reset(self):
        self.implementation.reset()

    def do_test(self, test):
        try:
            result = self.implementation.run_test(test)
            self.protocol.content_shell_errors.read_errors()
            return self.convert_result(test, result)
        except BaseException as exception:
            return _convert_exception(test, exception, self.protocol.content_shell_errors.read_errors())

    def screenshot(self, test, viewport_size, dpi, page_ranges):
        # Currently, the page size and DPI are hardcoded for print-reftests:
        #   https://chromium.googlesource.com/chromium/src/+/4e1b7bc33d42b401d7d9ad1dcba72883add3e2af/content/web_test/renderer/test_runner.cc#100
        # Content shell has an internal `window.testRunner.setPrintingSize(...)`
        # API, but it's not callable with protocol mode.
        assert dpi is None
        command = self.test_url(test)
        if self.is_print:
            # Currently, `content_shell` uses the expected image hash to avoid
            # dumping a matching image as an optimization. In Chromium, the
            # hash can be computed from an expected screenshot checked into the
            # source tree (i.e., without looking at a reference). This is not
            # possible in `wpt`, so pass an empty hash here to force a dump.
            command += "''print"

        _, image = self.protocol.content_shell_test.do_test(command,
                                                            timeout_for_test(self, test))
        if not image:
            return False, ("ERROR", self.protocol.content_shell_errors.read_errors())
        return True, b64encode(image).decode()


class ContentShellPrintRefTestExecutor(ContentShellRefTestExecutor):
    is_print = True


class ContentShellTestharnessExecutor(TestharnessExecutor, _SanitizerMixin):  # type: ignore
    # Chromium's `testdriver-vendor.js` partially implements testdriver support
    # with internal APIs [1].
    #
    # [1]: https://chromium.googlesource.com/chromium/src/+/HEAD/docs/testing/writing_web_tests.md#Relying-on-Blink_Specific-Testing-APIs
    supports_testdriver = True

    def __init__(self, logger, browser, server_config, timeout_multiplier=1, debug_info=None,
            **kwargs):
        super().__init__(logger, browser, server_config, timeout_multiplier, debug_info, **kwargs)
        self.protocol = ContentShellProtocol(self, browser)

    def do_test(self, test):
        try:
            text, _ = self.protocol.content_shell_test.do_test(self.test_url(test),
                                                               timeout_for_test(self, test))
            errors = self.protocol.content_shell_errors.read_errors()
            if not text:
                return (test.result_cls("ERROR", errors), [])

            result_url, status, message, stack, subtest_results = json.loads(text)
            if result_url != test.url:
                # Suppress `convert_result`'s URL validation.
                # See `testharnessreport-content-shell.js` for details.
                self.logger.warning('Got results from %s, expected %s' % (result_url, test.url))
                self.logger.warning('URL mismatch may be a false positive '
                                    'if the test navigates')
                result_url = test.url
            raw_result = result_url, status, message, stack, subtest_results
            return self.convert_result(test, raw_result)
        except BaseException as exception:
            return _convert_exception(test, exception, self.protocol.content_shell_errors.read_errors())
