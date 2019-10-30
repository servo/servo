import abc
import errno
import os
import platform
import socket
import time
import traceback

import mozprocess


__all__ = ["SeleniumServer", "ChromeDriverServer", "CWTChromeDriverServer",
           "EdgeChromiumDriverServer", "OperaDriverServer", "GeckoDriverServer",
           "InternetExplorerDriverServer", "EdgeDriverServer",
           "ServoDriverServer", "WebKitDriverServer", "WebDriverServer"]


class WebDriverServer(object):
    __metaclass__ = abc.ABCMeta

    default_base_path = "/"

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

        self._port = port
        self._cmd = None
        self._args = args if args is not None else []
        self._proc = None

    @abc.abstractmethod
    def make_command(self):
        """Returns the full command for starting the server process as a list."""

    def start(self, block=False):
        try:
            self._run(block)
        except KeyboardInterrupt:
            self.stop()

    def _run(self, block):
        self._cmd = self.make_command()
        self._proc = mozprocess.ProcessHandler(
            self._cmd,
            processOutputLine=self.on_output,
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

        self.logger.debug(
            "Waiting for WebDriver to become accessible: %s" % self.url)
        try:
            wait_for_service((self.host, self.port))
        except Exception:
            self.logger.error(
                "WebDriver was not accessible "
                "within the timeout:\n%s" % traceback.format_exc())
            raise

        if block:
            self._proc.wait()

    def stop(self, force=False):
        self.logger.debug("Stopping WebDriver")
        if self.is_alive:
            kill_result = self._proc.kill()
            if force and kill_result != 0:
                return self._proc.kill(9)
            return kill_result
        return not self.is_alive

    @property
    def is_alive(self):
        return hasattr(self._proc, "proc") and self._proc.poll() is None

    def on_output(self, line):
        self.logger.process_output(self.pid,
                                   line.decode("utf8", "replace"),
                                   command=" ".join(self._cmd))

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
    def __init__(self, logger, binary="chromedriver", port=None,
                 base_path="", args=None):
        WebDriverServer.__init__(
            self, logger, binary, port=port, base_path=base_path, args=args)

    def make_command(self):
        return [self.binary,
                cmd_arg("port", str(self.port)),
                cmd_arg("url-base", self.base_path) if self.base_path else ""] + self._args

class CWTChromeDriverServer(WebDriverServer):
    def __init__(self, logger, binary, port=None, args=None):
        WebDriverServer.__init__(self, logger, binary, port=port, args=args)

    def make_command(self):
        return [self.binary,
                "--port=%s" % str(self.port)] + self._args

class EdgeChromiumDriverServer(WebDriverServer):
    def __init__(self, logger, binary="msedgedriver", port=None,
                 base_path="", args=None):
        WebDriverServer.__init__(
            self, logger, binary, port=port, base_path=base_path, args=args)

    def make_command(self):
        return [self.binary,
                cmd_arg("port", str(self.port)),
                cmd_arg("url-base", self.base_path) if self.base_path else ""] + self._args

class EdgeDriverServer(WebDriverServer):
    def __init__(self, logger, binary="microsoftwebdriver.exe", port=None,
                 base_path="", host="localhost", args=None):
        WebDriverServer.__init__(
            self, logger, binary, host=host, port=port, args=args)

    def make_command(self):
        return [self.binary,
                "--port=%s" % str(self.port)] + self._args

class OperaDriverServer(ChromeDriverServer):
    def __init__(self, logger, binary="operadriver", port=None,
                 base_path="", args=None):
        ChromeDriverServer.__init__(
            self, logger, binary, port=port, base_path=base_path, args=args)


class InternetExplorerDriverServer(WebDriverServer):
    def __init__(self, logger, binary="IEDriverServer.exe", port=None,
                 base_path="", host="localhost", args=None):
        WebDriverServer.__init__(
            self, logger, binary, host=host, port=port, args=args)

    def make_command(self):
        return [self.binary,
                "--port=%s" % str(self.port)] + self._args


class GeckoDriverServer(WebDriverServer):
    def __init__(self, logger, marionette_port=2828, binary="geckodriver",
                 host="127.0.0.1", port=None, args=None):
        env = os.environ.copy()
        env["RUST_BACKTRACE"] = "1"
        WebDriverServer.__init__(self, logger, binary, host=host, port=port, env=env, args=args)
        self.marionette_port = marionette_port

    def make_command(self):
        return [self.binary,
                "--marionette-port", str(self.marionette_port),
                "--host", self.host,
                "--port", str(self.port)] + self._args


class SafariDriverServer(WebDriverServer):
    def __init__(self, logger, binary="safaridriver", port=None, args=None):
        WebDriverServer.__init__(
            self, logger, binary, port=port, args=args)

    def make_command(self):
        return [self.binary,
                "--port=%s" % str(self.port)] + self._args


class ServoDriverServer(WebDriverServer):
    def __init__(self, logger, binary="servo", binary_args=None, host="127.0.0.1",
                 port=None, args=None):
        env = os.environ.copy()
        env["RUST_BACKTRACE"] = "1"
        WebDriverServer.__init__(self, logger, binary, host=host, port=port, env=env, args=args)
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
    def __init__(self, logger, binary=None, port=None, args=None):
        WebDriverServer.__init__(self, logger, binary, port=port, args=args)

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
        except socket.error:
            continue
        else:
            return s.getsockname()[1]
        finally:
            s.close()


def wait_for_service(addr, timeout=60):
    """Waits until network service given as a tuple of (host, port) becomes
    available or the `timeout` duration is reached, at which point
    ``socket.error`` is raised."""
    end = time.time() + timeout
    while end > time.time():
        so = socket.socket()
        try:
            so.connect(addr)
        except socket.timeout:
            pass
        except socket.error as e:
            if e[0] != errno.ECONNREFUSED:
                raise
        else:
            return True
        finally:
            so.close()
        time.sleep(0.5)
    raise socket.error("Service is unavailable: %s:%i" % addr)
