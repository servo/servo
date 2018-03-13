import json
import os
import multiprocessing
import signal
import socket
import sys
import time

from mozlog import get_default_logger, handlers, proxy

from wptlogging import LogLevelRewriter
from wptserve.handlers import StringHandler

here = os.path.split(__file__)[0]
repo_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir, os.pardir))

serve = None
sslutils = None


def do_delayed_imports(logger, test_paths):
    global serve, sslutils

    serve_root = serve_path(test_paths)
    sys.path.insert(0, serve_root)

    failed = []

    try:
        from tools.serve import serve
    except ImportError:
        failed.append("serve")

    try:
        import sslutils
    except ImportError:
        failed.append("sslutils")

    if failed:
        logger.critical(
            "Failed to import %s. Ensure that tests path %s contains web-platform-tests" %
            (", ".join(failed), serve_root))
        sys.exit(1)


def serve_path(test_paths):
    return test_paths["/"]["tests_path"]


def get_ssl_kwargs(**kwargs):
    if kwargs["ssl_type"] == "openssl":
        args = {"openssl_binary": kwargs["openssl_binary"]}
    elif kwargs["ssl_type"] == "pregenerated":
        args = {"host_key_path": kwargs["host_key_path"],
                "host_cert_path": kwargs["host_cert_path"],
                "ca_cert_path": kwargs["ca_cert_path"]}
    else:
        args = {}
    return args


def ssl_env(logger, **kwargs):
    ssl_env_cls = sslutils.environments[kwargs["ssl_type"]]
    return ssl_env_cls(logger, **get_ssl_kwargs(**kwargs))


class TestEnvironmentError(Exception):
    pass


class TestEnvironment(object):
    def __init__(self, test_paths, ssl_env, pause_after_test, debug_info, options, env_extras):
        """Context manager that owns the test environment i.e. the http and
        websockets servers"""
        self.test_paths = test_paths
        self.ssl_env = ssl_env
        self.server = None
        self.config = None
        self.pause_after_test = pause_after_test
        self.test_server_port = options.pop("test_server_port", True)
        self.debug_info = debug_info
        self.options = options if options is not None else {}

        self.cache_manager = multiprocessing.Manager()
        self.stash = serve.stash.StashServer()
        self.env_extras = env_extras
        self.env_extras_cms = None


    def __enter__(self):
        self.stash.__enter__()
        self.ssl_env.__enter__()
        self.cache_manager.__enter__()

        self.config = self.load_config()
        self.setup_server_logging()
        ports = serve.get_ports(self.config, self.ssl_env)
        self.config = serve.normalise_config(self.config, ports)

        assert self.env_extras_cms is None, (
            "A TestEnvironment object cannot be nested")

        self.env_extras_cms = []

        for env in self.env_extras:
            cm = env(self.options, self.config)
            cm.__enter__()
            self.env_extras_cms.append(cm)

        self.servers = serve.start(self.config,
                                   self.ssl_env,
                                   self.get_routes())
        if self.options.get("supports_debugger") and self.debug_info and self.debug_info.interactive:
            self.ignore_interrupts()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.process_interrupts()

        for scheme, servers in self.servers.iteritems():
            for port, server in servers:
                server.kill()
        for cm in self.env_extras_cms:
            cm.__exit__(exc_type, exc_val, exc_tb)

        self.env_extras_cms = None

        self.cache_manager.__exit__(exc_type, exc_val, exc_tb)
        self.ssl_env.__exit__(exc_type, exc_val, exc_tb)
        self.stash.__exit__()

    def ignore_interrupts(self):
        signal.signal(signal.SIGINT, signal.SIG_IGN)

    def process_interrupts(self):
        signal.signal(signal.SIGINT, signal.SIG_DFL)

    def load_config(self):
        default_config_path = os.path.join(serve_path(self.test_paths), "config.default.json")
        local_config = {
            "ports": {
                "http": [8000, 8001],
                "https": [8443],
                "ws": [8888]
            },
            "check_subdomains": False,
            "ssl": {}
        }

        if "browser_host" in self.options:
            local_config["browser_host"] = self.options["browser_host"]

        if "bind_address" in self.options:
            local_config["bind_address"] = self.options["bind_address"]

        with open(default_config_path) as f:
            default_config = json.load(f)

        local_config["server_host"] = self.options.get("server_host", None)
        local_config["ssl"]["encrypt_after_connect"] = self.options.get("encrypt_after_connect", False)

        config = serve.merge_json(default_config, local_config)
        config["doc_root"] = serve_path(self.test_paths)

        if not self.ssl_env.ssl_enabled:
            config["ports"]["https"] = [None]

        host = config["browser_host"]
        hosts = [host]
        hosts.extend("%s.%s" % (item[0], host) for item in serve.get_subdomains(host).values())
        key_file, certificate = self.ssl_env.host_cert_path(hosts)

        config["key_file"] = key_file
        config["certificate"] = certificate

        serve.set_computed_defaults(config)

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
                 {"output": self.pause_after_test}, "text/javascript;charset=utf8",
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
        route_builder.add_handler(b"GET", b"/resources/testdriver.js",
                                  StringHandler(data, "text/javascript"))

        for url_base, paths in self.test_paths.iteritems():
            if url_base == "/":
                continue
            route_builder.add_mount_point(url_base, paths["tests_path"])

        if "/" not in self.test_paths:
            del route_builder.mountpoint_routes["/"]

        return route_builder.get_routes()

    def ensure_started(self):
        # Pause for a while to ensure that the server has a chance to start
        for _ in xrange(20):
            failed = self.test_servers()
            if not failed:
                return
            time.sleep(0.5)
        raise EnvironmentError("Servers failed to start: %s" %
                               ", ".join("%s:%s" % item for item in failed))

    def test_servers(self):
        failed = []
        host = self.config["server_host"]
        for scheme, servers in self.servers.iteritems():
            for port, server in servers:
                if self.test_server_port:
                    s = socket.socket()
                    try:
                        s.connect((host, port))
                    except socket.error:
                        failed.append((host, port))
                    finally:
                        s.close()

                if not server.is_alive():
                    failed.append((scheme, port))
        return failed
