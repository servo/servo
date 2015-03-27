# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import errno
import socket
import time
import traceback
import urlparse

import mozprocess

from .base import get_free_port, cmd_arg


__all__ = ["SeleniumLocalServer", "ChromedriverLocalServer"]


class LocalServer(object):
    used_ports = set()
    default_endpoint = "/"

    def __init__(self, logger, binary, port=None, endpoint=None):
        self.logger = logger
        self.binary = binary
        self.port = port
        self.endpoint = endpoint or self.default_endpoint

        if self.port is None:
            self.port = get_free_port(4444, exclude=self.used_ports)
        self.used_ports.add(self.port)
        self.url = "http://127.0.0.1:%i%s" % (self.port, self.endpoint)

        self.proc, self.cmd = None, None

    def start(self):
        self.proc = mozprocess.ProcessHandler(
            self.cmd, processOutputLine=self.on_output)
        try:
            self.proc.run()
        except OSError as e:
            if e.errno == errno.ENOENT:
                raise IOError(
                    "chromedriver executable not found: %s" % self.binary)
            raise

        self.logger.debug(
            "Waiting for server to become accessible: %s" % self.url)
        surl = urlparse.urlparse(self.url)
        addr = (surl.hostname, surl.port)
        try:
            wait_service(addr)
        except:
            self.logger.error(
                "Server was not accessible within the timeout:\n%s" % traceback.format_exc())
            raise
        else:
            self.logger.info("Server listening on port %i" % self.port)

    def stop(self):
        if hasattr(self.proc, "proc"):
            self.proc.kill()

    def is_alive(self):
        if hasattr(self.proc, "proc"):
            exitcode = self.proc.poll()
            return exitcode is None
        return False

    def on_output(self, line):
        self.logger.process_output(self.pid,
                                   line.decode("utf8", "replace"),
                                   command=" ".join(self.cmd))

    @property
    def pid(self):
        if hasattr(self.proc, "proc"):
            return self.proc.pid


class SeleniumLocalServer(LocalServer):
    default_endpoint = "/wd/hub"

    def __init__(self, logger, binary, port=None):
        LocalServer.__init__(self, logger, binary, port=port)
        self.cmd = ["java",
                    "-jar", self.binary,
                    "-port", str(self.port)]

    def start(self):
        self.logger.debug("Starting local Selenium server")
        LocalServer.start(self)

    def stop(self):
        LocalServer.stop(self)
        self.logger.info("Selenium server stopped listening")


class ChromedriverLocalServer(LocalServer):
    default_endpoint = "/wd/hub"

    def __init__(self, logger, binary="chromedriver", port=None, endpoint=None):
        LocalServer.__init__(self, logger, binary, port=port, endpoint=endpoint)
        # TODO: verbose logging
        self.cmd = [self.binary,
                    cmd_arg("port", str(self.port)) if self.port else "",
                    cmd_arg("url-base", self.endpoint) if self.endpoint else ""]

    def start(self):
        self.logger.debug("Starting local chromedriver server")
        LocalServer.start(self)

    def stop(self):
        LocalServer.stop(self)
        self.logger.info("chromedriver server stopped listening")


def wait_service(addr, timeout=15):
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
