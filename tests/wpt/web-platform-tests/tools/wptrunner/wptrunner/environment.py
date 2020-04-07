import json
import os
import multiprocessing
import signal
import socket
import sys
import time
from six import iteritems

from mozlog import get_default_logger, handlers, proxy

from .wptlogging import LogLevelRewriter

here = os.path.split(__file__)[0]
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
    return test_paths["/"]["tests_path"]


class TestEnvironmentError(Exception):
    pass


class TestEnvironment(object):
    """Context manager that owns the test environment i.e. the http and
    websockets servers"""
    def __init__(self, test_paths, testharness_timeout_multipler, pause_after_test, debug_info, options, ssl_config, env_extras):
        self.test_paths = test_paths
        self.server = None
        self.config_ctx = None
        self.config = None
        self.testharness_timeout_multipler = testharness_timeout_multipler
        self.pause_after_test = pause_after_test
        self.test_server_port = options.pop("test_server_port", True)
        self.debug_info = debug_info
        self.options = options if options is not None else {}

        self.cache_manager = multiprocessing.Manager()
        self.stash = serve.stash.StashServer()
        self.env_extras = env_extras
        self.env_extras_cms = None
        self.ssl_config = ssl_config

    def __enter__(self):
        self.config_ctx = self.build_config()

        self.config = self.config_ctx.__enter__()

        self.stash.__enter__()
        self.cache_manager.__enter__()

        self.setup_server_logging()

        assert self.env_extras_cms is None, (
            "A TestEnvironment object cannot be nested")

        self.env_extras_cms = []

        for env in self.env_extras:
            cm = env(self.options, self.config)
            cm.__enter__()
            self.env_extras_cms.append(cm)

        self.servers = serve.start(self.config,
                                   self.get_routes())

        if self.options.get("supports_debugger") and self.debug_info and self.debug_info.interactive:
            self.ignore_interrupts()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.process_interrupts()

        for scheme, servers in iteritems(self.servers):
            for port, server in servers:
                server.kill()
        for cm in self.env_extras_cms:
            cm.__exit__(exc_type, exc_val, exc_tb)

        self.env_extras_cms = None

        self.cache_manager.__exit__(exc_type, exc_val, exc_tb)
        self.stash.__exit__()
        self.config_ctx.__exit__(exc_type, exc_val, exc_tb)

    def ignore_interrupts(self):
        signal.signal(signal.SIGINT, signal.SIG_IGN)

    def process_interrupts(self):
        signal.signal(signal.SIGINT, signal.SIG_DFL)

    def build_config(self):
        override_path = os.path.join(serve_path(self.test_paths), "config.json")

        config = serve.ConfigBuilder()

        config.ports = {
            "http": [8000, 8001],
            "https": [8443],
            "ws": [8888],
            "wss": [8889],
            "h2": [9000],
        }

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

        return config

    def setup_server_logging(self):
        server_logger = get_default_logger(component="wptserve")
        assert server_logger is not None
        log_filter = handlers.LogLevelFilter(lambda x:x, "info")
        # Downgrade errors to warnings for the server
        log_filter = LogLevelRewriter(log_filter, ["error"], "warning")
        server_logger.component_filter = log_filter

        server_logger = proxy.QueuedProxyLogger(server_logger)

        try:
            #Set as the default logger for wptserve
            serve.set_logger(server_logger)
            serve.logger = server_logger
        except Exception:
            # This happens if logging has already been set up for wptserve
            pass

    def get_routes(self):
        route_builder = serve.RoutesBuilder()

        for path, format_args, content_type, route in [
                ("testharness_runner.html", {}, "text/html", "/testharness_runner.html"),
                (self.options.get("testharnessreport", "testharnessreport.js"),
                 {"output": self.pause_after_test,
                  "timeout_multiplier": self.testharness_timeout_multipler,
                  "explicit_timeout": "true" if self.debug_info is not None else "false"},
                 "text/javascript;charset=utf8",
                 "/resources/testharnessreport.js")]:
            path = os.path.normpath(os.path.join(here, path))
            # Note that .headers. files don't apply to static routes, so we need to
            # readd any static headers here.
            headers = {"Cache-Control": "max-age=3600"}
            route_builder.add_static(path, format_args, content_type, route,
                                     headers=headers)

        data = b""
        with open(os.path.join(repo_root, "resources", "testdriver.js"), "rb") as fp:
            data += fp.read()
        with open(os.path.join(here, "testdriver-extra.js"), "rb") as fp:
            data += fp.read()
        route_builder.add_handler("GET", "/resources/testdriver.js",
                                  StringHandler(data, "text/javascript"))

        for url_base, paths in iteritems(self.test_paths):
            if url_base == "/":
                continue
            route_builder.add_mount_point(url_base, paths["tests_path"])

        if "/" not in self.test_paths:
            del route_builder.mountpoint_routes["/"]

        return route_builder.get_routes()

    def ensure_started(self):
        # Pause for a while to ensure that the server has a chance to start
        total_sleep_secs = 30
        each_sleep_secs = 0.5
        end_time = time.time() + total_sleep_secs
        while time.time() < end_time:
            failed, pending = self.test_servers()
            if failed:
                break
            if not pending:
                return
            time.sleep(each_sleep_secs)
        raise EnvironmentError("Servers failed to start: %s" %
                               ", ".join("%s:%s" % item for item in failed))

    def test_servers(self):
        failed = []
        pending = []
        host = self.config["server_host"]
        for scheme, servers in iteritems(self.servers):
            for port, server in servers:
                if not server.is_alive():
                    failed.append((scheme, port))

        if not failed and self.test_server_port:
            for scheme, servers in iteritems(self.servers):
                for port, server in servers:
                    s = socket.socket()
                    s.settimeout(0.1)
                    try:
                        s.connect((host, port))
                    except socket.error:
                        pending.append((host, port))
                    finally:
                        s.close()

        return failed, pending
