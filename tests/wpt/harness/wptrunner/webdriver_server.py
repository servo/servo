# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import abc
import errno
import os
import platform
import socket
import threading
import time
import traceback
import urlparse

import mozprocess


__all__ = ["SeleniumServer", "ChromeDriverServer",
           "GeckoDriverServer", "WebDriverServer"]


class WebDriverServer(object):
    __metaclass__ = abc.ABCMeta

    default_base_path = "/"
    _used_ports = set()

    def __init__(self, logger, binary, host="127.0.0.1", port=None,
                 base_path="", env=None):
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
        self._proc = None

    @abc.abstractmethod
    def make_command(self):
        """Returns the full command for starting the server process as a list."""

    def start(self, block=True):
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

        try:
            self._proc.run()
        except OSError as e:
            if e.errno == errno.ENOENT:
                raise IOError(
                    "WebDriver HTTP server executable not found: %s" % self.binary)
            raise

        self.logger.debug(
            "Waiting for server to become accessible: %s" % self.url)
        try:
            wait_for_service((self.host, self.port))
        except:
            self.logger.error(
                "WebDriver HTTP server was not accessible "
                "within the timeout:\n%s" % traceback.format_exc())
            raise

        if block:
            self._proc.wait()

    def stop(self):
        if self.is_alive:
            return self._proc.kill()
        return not self.is_alive

    @property
    def is_alive(self):
        return (self._proc is not None and
                self._proc.proc is not None and
                self._proc.poll() is None)

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
            self._port = self._find_next_free_port()
        return self._port

    @staticmethod
    def _find_next_free_port():
        port = get_free_port(4444, exclude=WebDriverServer._used_ports)
        WebDriverServer._used_ports.add(port)
        return port


class SeleniumServer(WebDriverServer):
    default_base_path = "/wd/hub"

    def make_command(self):
        return ["java", "-jar", self.binary, "-port", str(self.port)]


class ChromeDriverServer(WebDriverServer):
    default_base_path = "/wd/hub"

    def __init__(self, logger, binary="chromedriver", port=None,
                 base_path=""):
        WebDriverServer.__init__(
            self, logger, binary, port=port, base_path=base_path)

    def make_command(self):
        return [self.binary,
                cmd_arg("port", str(self.port)),
                cmd_arg("url-base", self.base_path) if self.base_path else ""]


class GeckoDriverServer(WebDriverServer):
    def __init__(self, logger, marionette_port=2828, binary="wires",
                 host="127.0.0.1", port=None):
        env = os.environ.copy()
        env["RUST_BACKTRACE"] = "1"
        WebDriverServer.__init__(self, logger, binary, host=host, port=port, env=env)
        self.marionette_port = marionette_port

    def make_command(self):
        return [self.binary,
                "--connect-existing",
                "--marionette-port", str(self.marionette_port),
                "--webdriver-host", self.host,
                "--webdriver-port", str(self.port)]


def cmd_arg(name, value=None):
    prefix = "-" if platform.system() == "Windows" else "--"
    rv = prefix + name
    if value is not None:
        rv += "=" + value
    return rv


def get_free_port(start_port, exclude=None):
    """Get the first port number after start_port (inclusive) that is
    not currently bound.

    :param start_port: Integer port number at which to start testing.
    :param exclude: Set of port numbers to skip"""
    port = start_port
    while True:
        if exclude and port in exclude:
            port += 1
            continue
        s = socket.socket()
        try:
            s.bind(("127.0.0.1", port))
        except socket.error:
            port += 1
        else:
            return port
        finally:
            s.close()


def wait_for_service(addr, timeout=15):
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
