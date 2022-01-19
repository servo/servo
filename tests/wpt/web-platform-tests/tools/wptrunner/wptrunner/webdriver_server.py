import abc
import errno
import os
import platform
import socket
import time
import traceback
from typing import ClassVar, Type

import mozprocess

from .browsers.base import OutputHandler


__all__ = ["SeleniumServer", "ChromeDriverServer", "CWTChromeDriverServer",
           "EdgeChromiumDriverServer", "OperaDriverServer",
           "InternetExplorerDriverServer", "EdgeDriverServer",
           "ServoDriverServer", "WebKitDriverServer", "WebDriverServer"]


class WebDriverServer(object):
    __metaclass__ = abc.ABCMeta

    default_base_path = "/"
    output_handler_cls = OutputHandler  # type: ClassVar[Type[OutputHandler]]

    def __init__(self, logger, binary, host="127.0.0.1", port=None,
                 base_path="", env=None, args=None):
        if binary is None:
            raise ValueError("WebDriver server binary must be given "
                             "to --webdriver-binary argument")

        self.logger = logger
        self.binary = binary
        self.host = host

        if base_path == "":
            self.base_path = self.default_base_path
        else:
            self.base_path = base_path
        self.env = os.environ.copy() if env is None else env

        self._output_handler = None
        self._port = port
        self._cmd = None
        self._args = args if args is not None else []
        self._proc = None

    @abc.abstractmethod
    def make_command(self):
        """Returns the full command for starting the server process as a list."""

    def start(self,
              block=False,
              output_handler_kwargs=None,
              output_handler_start_kwargs=None):
        try:
            self._run(block, output_handler_kwargs, output_handler_start_kwargs)
        except KeyboardInterrupt:
            self.stop()

    def _run(self, block, output_handler_kwargs, output_handler_start_kwargs):
        if output_handler_kwargs is None:
            output_handler_kwargs = {}
        if output_handler_start_kwargs is None:
            output_handler_start_kwargs = {}
        self._cmd = self.make_command()
        self._output_handler = self.output_handler_cls(self.logger,
                                                       self._cmd,
                                                       **output_handler_kwargs)
        self._proc = mozprocess.ProcessHandler(
            self._cmd,
            processOutputLine=self._output_handler,
            env=self.env,
            storeOutput=False)

        self.logger.debug("Starting WebDriver: %s" % ' '.join(self._cmd))
        try:
            self._proc.run()
        except OSError as e:
            if e.errno == errno.ENOENT:
                raise IOError(
                    "WebDriver executable not found: %s" % self.binary)
            raise
        self._output_handler.after_process_start(self._proc.pid)

        self.logger.debug(
            "Waiting for WebDriver to become accessible: %s" % self.url)
        try:
            wait_for_service((self.host, self.port))
        except Exception:
            self.logger.error(
                "WebDriver was not accessible "
                "within the timeout:\n%s" % traceback.format_exc())
            raise
        self._output_handler.start(**output_handler_start_kwargs)
        if block:
            self._proc.wait()

    def stop(self, force=False):
        self.logger.debug("Stopping WebDriver")
        clean = True
        if self.is_alive():
            kill_result = self._proc.kill()
            if force and kill_result != 0:
                clean = False
                self._proc.kill(9)
        success = not self.is_alive()
        if success and self._output_handler is not None:
            # Only try to do output post-processing if we managed to shut down
            self._output_handler.after_process_stop(clean)
            self._output_handler = None
        return success

    def is_alive(self):
        return hasattr(self._proc, "proc") and self._proc.poll() is None

    @property
    def pid(self):
        if self._proc is not None:
            return self._proc.pid

    @property
    def url(self):
        return "http://%s:%i%s" % (self.host, self.port, self.base_path)

    @property
    def port(self):
        if self._port is None:
            self._port = get_free_port()
        return self._port


class SeleniumServer(WebDriverServer):
    default_base_path = "/wd/hub"

    def make_command(self):
        return ["java", "-jar", self.binary, "-port", str(self.port)] + self._args


class ChromeDriverServer(WebDriverServer):
    def make_command(self):
        return [self.binary,
                cmd_arg("port", str(self.port)),
                cmd_arg("url-base", self.base_path) if self.base_path else "",
                cmd_arg("enable-chrome-logs")] + self._args


class CWTChromeDriverServer(WebDriverServer):
    def make_command(self):
        return [self.binary,
                "--port=%s" % str(self.port)] + self._args


class EdgeChromiumDriverServer(WebDriverServer):
    def make_command(self):
        return [self.binary,
                cmd_arg("port", str(self.port)),
                cmd_arg("url-base", self.base_path) if self.base_path else ""] + self._args


class EdgeDriverServer(WebDriverServer):
    def make_command(self):
        return [self.binary,
                "--port=%s" % str(self.port)] + self._args


class OperaDriverServer(ChromeDriverServer):
    pass

class InternetExplorerDriverServer(WebDriverServer):
    def make_command(self):
        return [self.binary,
                "--port=%s" % str(self.port)] + self._args


class SafariDriverServer(WebDriverServer):
    def make_command(self):
        return [self.binary,
                "--port=%s" % str(self.port)] + self._args


class ServoDriverServer(WebDriverServer):
    def __init__(self, logger, binary="servo", binary_args=None, host="127.0.0.1",
                 port=None, env=None, args=None):
        env = env if env is not None else os.environ.copy()
        env["RUST_BACKTRACE"] = "1"
        WebDriverServer.__init__(self, logger, binary,
                                 host=host,
                                 port=port,
                                 env=env,
                                 args=args)
        self.binary_args = binary_args

    def make_command(self):
        command = [self.binary,
                   "--webdriver=%s" % self.port,
                   "--hard-fail",
                   "--headless"] + self._args
        if self.binary_args:
            command += self.binary_args
        return command


class WebKitDriverServer(WebDriverServer):
    def make_command(self):
        return [self.binary, "--port=%s" % str(self.port)] + self._args


def cmd_arg(name, value=None):
    prefix = "-" if platform.system() == "Windows" else "--"
    rv = prefix + name
    if value is not None:
        rv += "=" + value
    return rv


def get_free_port():
    """Get a random unbound port"""
    while True:
        s = socket.socket()
        try:
            s.bind(("127.0.0.1", 0))
        except OSError:
            continue
        else:
            return s.getsockname()[1]
        finally:
            s.close()


def wait_for_service(addr, timeout=60):
    """Waits until network service given as a tuple of (host, port) becomes
    available or the `timeout` duration is reached, at which point
    ``socket.timeout`` is raised."""
    end = time.time() + timeout
    while end > time.time():
        so = socket.socket()
        try:
            so.connect(addr)
        except socket.timeout:
            pass
        except OSError as e:
            if e.errno != errno.ECONNREFUSED:
                raise
        else:
            return True
        finally:
            so.close()
        time.sleep(0.5)
    raise socket.timeout("Service is unavailable: %s:%i" % addr)
