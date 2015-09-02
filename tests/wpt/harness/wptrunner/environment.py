# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import json
import os
import multiprocessing
import signal
import socket
import sys
import time

from mozlog import get_default_logger, handlers

from wptlogging import LogLevelRewriter

here = os.path.split(__file__)[0]

serve = None
sslutils = None


hostnames = ["web-platform.test",
             "www.web-platform.test",
             "www1.web-platform.test",
             "www2.web-platform.test",
             "xn--n8j6ds53lwwkrqhv28a.web-platform.test",
             "xn--lve-6lad.web-platform.test"]


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
        raise
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
    def __init__(self, test_paths, ssl_env, pause_after_test, debug_info, options):
        """Context manager that owns the test environment i.e. the http and
        websockets servers"""
        self.test_paths = test_paths
        self.ssl_env = ssl_env
        self.server = None
        self.config = None
        self.external_config = None
        self.pause_after_test = pause_after_test
        self.test_server_port = options.pop("test_server_port", True)
        self.debug_info = debug_info
        self.options = options if options is not None else {}

        self.cache_manager = multiprocessing.Manager()
        self.stash = serve.stash.StashServer()


    def __enter__(self):
        self.stash.__enter__()
        self.ssl_env.__enter__()
        self.cache_manager.__enter__()
        self.setup_server_logging()
        self.config = self.load_config()
        serve.set_computed_defaults(self.config)
        self.external_config, self.servers = serve.start(self.config, self.ssl_env,
                                                         self.get_routes())
        if self.options.get("supports_debugger") and self.debug_info and self.debug_info.interactive:
            self.ignore_interrupts()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.process_interrupts()
        for scheme, servers in self.servers.iteritems():
            for port, server in servers:
                server.kill()
        self.cache_manager.__exit__(exc_type, exc_val, exc_tb)
        self.ssl_env.__exit__(exc_type, exc_val, exc_tb)
        self.stash.__exit__()

    def ignore_interrupts(self):
        signal.signal(signal.SIGINT, signal.SIG_IGN)

    def process_interrupts(self):
        signal.signal(signal.SIGINT, signal.SIG_DFL)

    def load_config(self):
        default_config_path = os.path.join(serve_path(self.test_paths), "config.default.json")
        local_config_path = os.path.join(here, "config.json")

        with open(default_config_path) as f:
            default_config = json.load(f)

        with open(local_config_path) as f:
            data = f.read()
            local_config = json.loads(data % self.options)

        #TODO: allow non-default configuration for ssl

        local_config["external_host"] = self.options.get("external_host", None)
        local_config["ssl"]["encrypt_after_connect"] = self.options.get("encrypt_after_connect", False)

        config = serve.merge_json(default_config, local_config)
        config["doc_root"] = serve_path(self.test_paths)

        if not self.ssl_env.ssl_enabled:
            config["ports"]["https"] = [None]

        host = self.options.get("certificate_domain", config["host"])
        hosts = [host]
        hosts.extend("%s.%s" % (item[0], host) for item in serve.get_subdomains(host).values())
        key_file, certificate = self.ssl_env.host_cert_path(hosts)

        config["key_file"] = key_file
        config["certificate"] = certificate

        return config

    def setup_server_logging(self):
        server_logger = get_default_logger(component="wptserve")
        assert server_logger is not None
        log_filter = handlers.LogLevelFilter(lambda x:x, "info")
        # Downgrade errors to warnings for the server
        log_filter = LogLevelRewriter(log_filter, ["error"], "warning")
        server_logger.component_filter = log_filter

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
                 {"output": self.pause_after_test}, "text/javascript",
                 "/resources/testharnessreport.js")]:
            path = os.path.normpath(os.path.join(here, path))
            route_builder.add_static(path, format_args, content_type, route)

        for url_base, paths in self.test_paths.iteritems():
            if url_base == "/":
                continue
            route_builder.add_mount_point(url_base, paths["tests_path"])

        if "/" not in self.test_paths:
            del route_builder.mountpoint_routes["/"]

        return route_builder.get_routes()

    def ensure_started(self):
        # Pause for a while to ensure that the server has a chance to start
        time.sleep(2)
        for scheme, servers in self.servers.iteritems():
            for port, server in servers:
                if self.test_server_port:
                    s = socket.socket()
                    try:
                        s.connect((self.config["host"], port))
                    except socket.error:
                        raise EnvironmentError(
                            "%s server on port %d failed to start" % (scheme, port))
                    finally:
                        s.close()

                if not server.is_alive():
                    raise EnvironmentError("%s server on port %d failed to start" % (scheme, port))
