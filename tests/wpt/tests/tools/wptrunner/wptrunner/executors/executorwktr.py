# mypy: allow-untyped-defs

from .base import RefTestExecutor, RefTestImplementation, CrashtestExecutor, TestharnessExecutor
from .protocol import Protocol, ProtocolPart
from time import time
from queue import Empty
from base64 import b64encode
import json


class CrashError(BaseException):
    pass


def _read_line(io_queue, deadline=None, encoding=None, errors="strict", raise_crash=True, logger=None):
    """Reads a single line from the io queue. The read must succeed before `deadline` or
    a TimeoutError is raised. The line is returned as a bytestring or optionally with the
    specified `encoding`. If `raise_crash` is set, a CrashError is raised if the line
    happens to be a crash message.
    """
    current_time = time()

    if deadline and current_time > deadline:
        raise TimeoutError()

    try:
        line = io_queue.get(True, deadline - current_time if deadline else None)
        if raise_crash and line.startswith(b"#CRASHED"):
            raise CrashError()
    except Empty as e:
        logger.debug(f"got empty line with {time() - deadline} remaining")
        raise TimeoutError() from e

    return line.decode(encoding, errors) if encoding else line


class WKTRTestPart(ProtocolPart):
    """This protocol part is responsible for running tests via WebKitTestRunner's protocol mode.
    """
    name = "wktr_test"
    eof_marker = '#EOF\n'  # Marker sent by wktr after blocks.

    def __init__(self, parent):
        super().__init__(parent)
        self.stdout_queue = parent.browser.stdout_queue
        self.stdin_queue = parent.browser.stdin_queue

    def do_test(self, command, timeout=None):
        """Send a command to wktr and return the resulting outputs.

        A command consists of a URL to navigate to, followed by an optional options; see
        https://github.com/WebKit/WebKit/blob/main/Tools/TestRunnerShared/TestCommand.cpp.

        """
        self._send_command(command + "'--timeout'%d" % (timeout * 1000))

        deadline = time() + timeout if timeout else None
        # The first block can also contain audio data but not in WPT.
        text = self._read_block(deadline)
        image = self._read_block(deadline)

        return text, image

    def _send_command(self, command):
        """Sends a single `command`, i.e. a URL to open, to wktr.
        """
        self.stdin_queue.put((command + "\n").encode("utf-8"))

    def _read_block(self, deadline=None):
        """Tries to read a single block of content from stdout before the `deadline`.
        """
        while True:
            line = _read_line(self.stdout_queue, deadline, "latin-1", logger=self.logger).rstrip()

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
            line = _read_line(self.stdout_queue, deadline, "utf-8", "replace", False, logger=self.logger)

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
        content_length_line = _read_line(self.stdout_queue, deadline, "utf-8", logger=self.logger)
        assert content_length_line.startswith("Content-Length:")
        content_length = int(content_length_line[15:])

        result = bytearray()

        while True:
            line = _read_line(self.stdout_queue, deadline, raise_crash=False, logger=self.logger)
            excess = len(line) + len(result) - content_length

            if excess > 0:
                # This is the line that contains the EOF marker.
                assert excess == len(self.eof_marker)
                result += line[:-excess]
                break

            result += line

        return result


class WKTRErrorsPart(ProtocolPart):
    """This protocol part is responsible for collecting the errors reported by wktr.
    """
    name = "wktr_errors"

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
            result += _read_line(self.stderr_queue, None, "utf-8", "replace", False, logger=self.logger)

        return result


class WKTRProtocol(Protocol):
    implements = [WKTRTestPart, WKTRErrorsPart]

    def connect(self):
        pass

    def after_connect(self):
        pass

    def teardown(self):
        # Close the queue properly to avoid broken pipe spam in the log.
        self.browser.stdin_queue.close()
        self.browser.stdin_queue.join_thread()

    def is_alive(self):
        """Checks if wktr is alive by determining if the IO pipes are still
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
    raise exception


class WKTRRefTestExecutor(RefTestExecutor):
    def __init__(self, logger, browser, server_config, timeout_multiplier=1, screenshot_cache=None,
            debug_info=None, reftest_screenshot="unexpected", **kwargs):
        super().__init__(logger, browser, server_config, timeout_multiplier, screenshot_cache,
                debug_info, reftest_screenshot, **kwargs)
        self.implementation = RefTestImplementation(self)
        self.protocol = WKTRProtocol(self, browser)

    def reset(self):
        self.implementation.reset()

    def do_test(self, test):
        try:
            result = self.implementation.run_test(test)
            self.protocol.wktr_errors.read_errors()
            return self.convert_result(test, result)
        except BaseException as exception:
            return _convert_exception(test, exception, self.protocol.wktr_errors.read_errors())

    def screenshot(self, test, viewport_size, dpi, page_ranges):
        assert dpi is None
        command = self.test_url(test)
        command += "'--pixel-test'"
        assert not self.is_print
        _, image = self.protocol.wktr_test.do_test(
            command, test.timeout * self.timeout_multiplier)

        if not image:
            return False, ("ERROR", self.protocol.wktr_errors.read_errors())

        return True, b64encode(image).decode()

    def wait(self):
        return


class WKTRCrashtestExecutor(CrashtestExecutor):
    def __init__(self, logger, browser, server_config, timeout_multiplier=1, debug_info=None,
            **kwargs):
        super().__init__(logger, browser, server_config, timeout_multiplier, debug_info, **kwargs)
        self.protocol = WKTRProtocol(self, browser)

    def do_test(self, test):
        try:
            _ = self.protocol.wktr_test.do_test(self.test_url(test), test.timeout * self.timeout_multiplier)
            self.protocol.wktr_errors.read_errors()
            return self.convert_result(test, {"status": "PASS", "message": None})
        except BaseException as exception:
            return _convert_exception(test, exception, self.protocol.wktr_errors.read_errors())

    def wait(self):
        return


class WKTRTestharnessExecutor(TestharnessExecutor):
    def __init__(self, logger, browser, server_config, timeout_multiplier=1, debug_info=None,
            **kwargs):
        super().__init__(logger, browser, server_config, timeout_multiplier, debug_info, **kwargs)
        self.protocol = WKTRProtocol(self, browser)

    def do_test(self, test):
        try:
            text, _ = self.protocol.wktr_test.do_test(self.test_url(test),
                    test.timeout * self.timeout_multiplier)

            errors = self.protocol.wktr_errors.read_errors()
            if not text:
                return (test.result_cls("ERROR", errors), [])

            output = None
            output_prefix = "CONSOLE MESSAGE: WPTRUNNER OUTPUT:"

            for line in text.split("\n"):
                if line.startswith(output_prefix):
                    if output is None:
                        output = line[len(output_prefix):]
                    else:
                        return (test.result_cls("ERROR", "multiple wptrunner outputs"), [])

            if output is None:
                return (test.result_cls("ERROR", "no wptrunner output"), [])

            return self.convert_result(test, json.loads(output))
        except BaseException as exception:
            return _convert_exception(test, exception, self.protocol.wktr_errors.read_errors())

    def wait(self):
        return
