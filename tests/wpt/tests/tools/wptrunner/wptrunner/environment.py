# mypy: allow-untyped-defs

import contextlib
import errno
import json
import os
import signal
import socket
import sys
import time
from typing import Optional

import mozprocess
from mozlog import get_default_logger, handlers
from mozlog.structuredlog import StructuredLogger

from . import mpcontext
from .wptlogging import LogLevelRewriter, QueueHandler, LogQueueThread

here = os.path.dirname(__file__)
repo_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir, os.pardir))

sys.path.insert(0, repo_root)
from tools import localpaths  # noqa: F401

from wptserve.handlers import StringHandler

serve = None


def do_delayed_imports(logger, test_paths):
    global serve

    serve_root = serve_path(test_paths)
    sys.path.insert(0, serve_root)

    failed = []

    try:
        from tools.serve import serve
    except ImportError:
        failed.append("serve")

    if failed:
        logger.critical(
            "Failed to import %s. Ensure that tests path %s contains web-platform-tests" %
            (", ".join(failed), serve_root))
        sys.exit(1)


def serve_path(test_paths):
    return test_paths["/"].tests_path


def webtranport_h3_server_is_running(host, port, timeout):
    # TODO(bashi): Move the following import to the beginning of this file
    # once WebTransportH3Server is enabled by default.
    from webtransport.h3.webtransport_h3_server import server_is_running  # type: ignore
    return server_is_running(host, port, timeout)


class TestEnvironmentError(Exception):
    pass


def get_server_logger():
    logger = get_default_logger(component="wptserve")
    log_filter = handlers.LogLevelFilter(lambda x: x, "info")
    # Downgrade errors to warnings for the server
    log_filter = LogLevelRewriter(log_filter, ["error"], "warning")
    logger.component_filter = log_filter
    return logger


class ProxyLoggingContext:
    """Context manager object that handles setup and teardown of a log queue
    for handling logging messages from wptserve."""

    def __init__(self, logger):
        mp_context = mpcontext.get_context()
        self.log_queue = mp_context.Queue()
        self.logging_thread = LogQueueThread(self.log_queue, logger)
        self.logger_handler = QueueHandler(self.log_queue)

    def __enter__(self):
        self.logging_thread.start()
        return self.logger_handler

    def __exit__(self, *args):
        self.log_queue.put(None)
        # Wait for thread to shut down but not for too long since it's a daemon
        self.logging_thread.join(1)


class TestEnvironment:
    """Context manager that owns the test environment i.e. the http and
    websockets servers"""
    def __init__(self, test_paths, testharness_timeout_multipler,
                 pause_after_test, debug_test, debug_info, options, ssl_config, env_extras,
                 enable_webtransport=False, mojojs_path=None, inject_script=None,
                 suppress_handler_traceback=None, ws_extra=None):

        self.test_paths = test_paths
        self.server = None
        self.config_ctx = None
        self.config = None
        self.server_logger = get_server_logger()
        self.server_logging_ctx = ProxyLoggingContext(self.server_logger)
        self.testharness_timeout_multipler = testharness_timeout_multipler
        self.pause_after_test = pause_after_test
        self.debug_test = debug_test
        self.test_server_port = options.pop("test_server_port", True)
        self.debug_info = debug_info
        self.options = options if options is not None else {}

        mp_context = mpcontext.get_context()
        self._stack = contextlib.ExitStack()
        self.cache_manager = mp_context.Manager()
        self.stash = serve.stash.StashServer(mp_context=mp_context)
        self.env_extras = env_extras
        self.env_extras_cms = None
        self.ssl_config = ssl_config
        self.enable_webtransport = enable_webtransport
        self.mojojs_path = mojojs_path
        self.inject_script = inject_script
        self.suppress_handler_traceback = suppress_handler_traceback
        self.ws_extra = ws_extra

    def __enter__(self):
        server_log_handler = self._stack.enter_context(self.server_logging_ctx)
        self.config_ctx = self.build_config()

        self.config = self._stack.enter_context(self.config_ctx)

        self._stack.enter_context(self.stash)
        self._stack.enter_context(self.cache_manager)

        assert self.env_extras_cms is None, (
            "A TestEnvironment object cannot be nested")

        self.env_extras_cms = []

        for env in self.env_extras:
            cm = env(self.options, self.config)
            self._stack.enter_context(cm)
            self.env_extras_cms.append(cm)

        self.servers = serve.start(self.server_logger,
                                   self.config,
                                   self.get_routes(),
                                   mp_context=mpcontext.get_context(),
                                   log_handlers=[server_log_handler],
                                   webtransport_h3=self.enable_webtransport)

        if self.options.get("supports_debugger") and self.debug_info and self.debug_info.interactive:
            self._stack.enter_context(self.ignore_interrupts())
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        for servers in self.servers.values():
            for _, server in servers:
                server.request_shutdown()
        for servers in self.servers.values():
            for _, server in servers:
                server.wait()

        self._stack.__exit__(exc_type, exc_val, exc_tb)
        self.env_extras_cms = None

    @contextlib.contextmanager
    def ignore_interrupts(self):
        prev_handler = signal.signal(signal.SIGINT, signal.SIG_IGN)
        try:
            yield
        finally:
            signal.signal(signal.SIGINT, prev_handler)

    def build_config(self):
        override_path = os.path.join(serve_path(self.test_paths), "config.json")

        config = serve.ConfigBuilder(self.server_logger, ws_extra=self.ws_extra)

        ports = {
            "http": [8000, 8001],
            "http-private": [8002],
            "http-public": [8003],
            "https": [8443, 8444],
            "https-private": [8445],
            "https-public": [8446],
            "ws": [8888],
            "wss": [8889],
            "h2": [9000],
            "webtransport-h3": [11000],
        }
        config.ports = ports

        if os.path.exists(override_path):
            with open(override_path) as f:
                override_obj = json.load(f)
            config.update(override_obj)

        config.check_subdomains = False

        ssl_config = self.ssl_config.copy()
        ssl_config["encrypt_after_connect"] = self.options.get("encrypt_after_connect", False)
        config.ssl = ssl_config

        if "browser_host" in self.options:
            config.browser_host = self.options["browser_host"]

        if "bind_address" in self.options:
            config.bind_address = self.options["bind_address"]

        config.server_host = self.options.get("server_host", None)
        config.doc_root = serve_path(self.test_paths)
        config.inject_script = self.inject_script

        if self.suppress_handler_traceback is not None:
            config.logging["suppress_handler_traceback"] = self.suppress_handler_traceback

        return config

    def get_routes(self):
        route_builder = serve.get_route_builder(
            self.server_logger,
            self.config.aliases,
            self.config)

        for path, format_args, content_type, route in [
                ("testharness_runner.html", {}, "text/html", "/testharness_runner.html"),
                ("print_pdf_runner.html", {}, "text/html", "/print_pdf_runner.html"),
                (os.path.join(here, "..", "..", "third_party", "pdf_js", "pdf.js"), None,
                 "text/javascript", "/_pdf_js/pdf.js"),
                (os.path.join(here, "..", "..", "third_party", "pdf_js", "pdf.worker.js"), None,
                 "text/javascript", "/_pdf_js/pdf.worker.js"),
                (self.options.get("testharnessreport", "testharnessreport.js"),
                 {"output": self.pause_after_test,
                  "timeout_multiplier": self.testharness_timeout_multipler,
                  "explicit_timeout": "true" if self.debug_info is not None else "false",
                  "debug": "true" if self.debug_test else "false"},
                 "text/javascript;charset=utf8",
                 "/resources/testharnessreport.js")]:
            path = os.path.normpath(os.path.join(here, path))
            # Note that .headers. files don't apply to static routes, so we need to
            # readd any static headers here.
            headers = {"Cache-Control": "max-age=3600"}
            route_builder.add_static(path, format_args, content_type, route,
                                     headers=headers)

        route_builder.add_handler("GET", "/resources/testdriver.js", TestdriverLoader())

        for url_base, test_root in self.test_paths.items():
            if url_base == "/":
                continue
            route_builder.add_mount_point(url_base, test_root.tests_path)

        if "/" not in self.test_paths:
            del route_builder.mountpoint_routes["/"]

        if self.mojojs_path:
            route_builder.add_mount_point("/gen/", self.mojojs_path)

        return route_builder.get_routes()

    def ensure_started(self):
        # Pause for a while to ensure that the server has a chance to start
        total_sleep_secs = 60
        each_sleep_secs = 0.5
        end_time = time.time() + total_sleep_secs
        while time.time() < end_time:
            failed, pending = self.test_servers()
            if failed:
                break
            if not pending:
                return
            time.sleep(each_sleep_secs)
        if failed:
            failures = ", ".join(f"{scheme}:{port}" for scheme, port in failed)
            msg = f"Servers failed to start: {failures}"
        else:
            pending = ", ".join(f"{scheme}:{port}" for scheme, port in pending)
            msg = f"Timed out wait for servers to start: {pending}"
        raise OSError(msg)

    def test_servers(self):
        failed = []
        pending = []
        host = self.config["server_host"]
        for scheme, servers in self.servers.items():
            for port, server in servers:
                if not server.is_alive():
                    failed.append((scheme, port))

        if not failed and self.test_server_port:
            # The webtransport-h3 server test blocks (i.e., doesn't fail quickly
            # with "Connection refused" like the sockets do), so testing these
            # first improves the likelihood the non-webtransport-h3 servers are
            # ready by the time they're checked.
            for port, server in self.servers.get("webtransport-h3", []):
                if not webtranport_h3_server_is_running(host, port, timeout=5):
                    pending.append((host, port))

            for scheme, servers in self.servers.items():
                if scheme == "webtransport-h3":
                    continue
                for port, server in servers:
                    s = socket.socket()
                    s.settimeout(0.1)
                    try:
                        s.connect((host, port))
                    except OSError:
                        pending.append((scheme, port))
                    finally:
                        s.close()

        return failed, pending


class TestdriverLoader:
    """A special static handler for serving `/resources/testdriver.js`.

    This handler lazily reads `testdriver{,-extra}.js` so that wptrunner doesn't
    need to pass the entire file contents to child `wptserve` processes, which
    can slow `wptserve` startup by several seconds (crbug.com/1479850).
    """
    def __init__(self):
        self._handler = None

    def __call__(self, request, response):
        if not self._handler:
            data = b""
            with open(os.path.join(repo_root, "resources", "testdriver.js"), "rb") as fp:
                data += fp.read()
            with open(os.path.join(here, "testdriver-extra.js"), "rb") as fp:
                data += fp.read()
            self._handler = StringHandler(data, "text/javascript")
        return self._handler(request, response)


def wait_for_service(logger: StructuredLogger,
                     host: str,
                     port: int,
                     timeout: float = 60,
                     server_process: Optional[mozprocess.ProcessHandler] = None) -> bool:
    """Waits until network service given as a tuple of (host, port) becomes
    available, `timeout` duration is reached, or the `server_process` exits at
    which point ``socket.error`` is raised."""
    addr = (host, port)
    logger.debug(f"Trying to connect to {host}:{port}")
    end = time.time() + timeout
    while end > time.time():
        if server_process is not None and server_process.poll() is not None:
            returncode = server_process.poll()
            logger.debug(
                f"Server process {server_process.pid} exited with "
                f"{returncode}, giving up trying to connect"
            )
            break

        so = socket.socket()
        try:
            so.connect(addr)
        except socket.timeout:
            pass
        except OSError as e:
            if e.errno != errno.ECONNREFUSED:
                raise
        else:
            logger.debug(f"Connected to {host}:{port}")
            return True
        finally:
            so.close()
        time.sleep(0.5)
    raise OSError("Service is unavailable: %s:%i" % addr)
