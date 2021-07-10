# -*- coding: utf-8 -*-

from __future__ import print_function

import abc
import argparse
import importlib
import json
import logging
import multiprocessing
import os
import platform
import signal
import socket
import subprocess
import sys
import threading
import time
import traceback
import urllib
import uuid
from collections import defaultdict, OrderedDict
from itertools import chain, product

from localpaths import repo_root

from manifest.sourcefile import read_script_metadata, js_meta_re, parse_variants
from wptserve import server as wptserve, handlers
from wptserve import stash
from wptserve import config
from wptserve.logger import set_logger
from wptserve.handlers import filesystem_path, wrap_pipeline
from wptserve.utils import get_port, HTTPException, http2_compatible
from mod_pywebsocket import standalone as pywebsocket


EDIT_HOSTS_HELP = ("Please ensure all the necessary WPT subdomains "
                   "are mapped to a loopback device in /etc/hosts.\n"
                   "See https://web-platform-tests.org/running-tests/from-local-system.html#system-setup "
                   "for instructions.")


def replace_end(s, old, new):
    """
    Given a string `s` that ends with `old`, replace that occurrence of `old`
    with `new`.
    """
    assert s.endswith(old)
    return s[:-len(old)] + new


def domains_are_distinct(a, b):
    a_parts = a.split(".")
    b_parts = b.split(".")
    min_length = min(len(a_parts), len(b_parts))
    slice_index = -1 * min_length

    return a_parts[slice_index:] != b_parts[slice_index:]


class WrapperHandler(object):

    __meta__ = abc.ABCMeta

    headers = []

    def __init__(self, base_path=None, url_base="/"):
        self.base_path = base_path
        self.url_base = url_base
        self.handler = handlers.handler(self.handle_request)

    def __call__(self, request, response):
        self.handler(request, response)

    def handle_request(self, request, response):
        headers = self.headers + handlers.load_headers(
            request, self._get_filesystem_path(request))
        for header_name, header_value in headers:
            response.headers.set(header_name, header_value)

        self.check_exposure(request)

        path = self._get_path(request.url_parts.path, True)
        query = request.url_parts.query
        if query:
            query = "?" + query
        meta = "\n".join(self._get_meta(request))
        script = "\n".join(self._get_script(request))
        response.content = self.wrapper % {"meta": meta, "script": script, "path": path, "query": query}
        wrap_pipeline(path, request, response)

    def _get_path(self, path, resource_path):
        """Convert the path from an incoming request into a path corresponding to an "unwrapped"
        resource e.g. the file on disk that will be loaded in the wrapper.

        :param path: Path from the HTTP request
        :param resource_path: Boolean used to control whether to get the path for the resource that
                              this wrapper will load or the associated file on disk.
                              Typically these are the same but may differ when there are multiple
                              layers of wrapping e.g. for a .any.worker.html input the underlying disk file is
                              .any.js but the top level html file loads a resource with a
                              .any.worker.js extension, which itself loads the .any.js file.
                              If True return the path to the resource that the wrapper will load,
                              otherwise return the path to the underlying file on disk."""
        for item in self.path_replace:
            if len(item) == 2:
                src, dest = item
            else:
                assert len(item) == 3
                src = item[0]
                dest = item[2 if resource_path else 1]
            if path.endswith(src):
                path = replace_end(path, src, dest)
        return path

    def _get_filesystem_path(self, request):
        """Get the path of the underlying resource file on disk."""
        return self._get_path(filesystem_path(self.base_path, request, self.url_base), False)

    def _get_metadata(self, request):
        """Get an iterator over script metadata based on // META comments in the
        associated js file.

        :param request: The Request being processed.
        """
        path = self._get_filesystem_path(request)
        try:
            with open(path, "rb") as f:
                for key, value in read_script_metadata(f, js_meta_re):
                    yield key, value
        except IOError:
            raise HTTPException(404)

    def _get_meta(self, request):
        """Get an iterator over strings to inject into the wrapper document
        based on // META comments in the associated js file.

        :param request: The Request being processed.
        """
        for key, value in self._get_metadata(request):
            replacement = self._meta_replacement(key, value)
            if replacement:
                yield replacement

    def _get_script(self, request):
        """Get an iterator over strings to inject into the wrapper document
        based on // META comments in the associated js file.

        :param request: The Request being processed.
        """
        for key, value in self._get_metadata(request):
            replacement = self._script_replacement(key, value)
            if replacement:
                yield replacement

    @abc.abstractproperty
    def path_replace(self):
        # A list containing a mix of 2 item tuples with (input suffix, output suffix)
        # and 3-item tuples with (input suffix, filesystem suffix, resource suffix)
        # for the case where we want a different path in the generated resource to
        # the actual path on the filesystem (e.g. when there is another handler
        # that will wrap the file).
        return None

    @abc.abstractproperty
    def wrapper(self):
        # String template with variables path and meta for wrapper document
        return None

    @abc.abstractmethod
    def _meta_replacement(self, key, value):
        # Get the string to insert into the wrapper document, given
        # a specific metadata key: value pair.
        pass

    @abc.abstractmethod
    def check_exposure(self, request):
        # Raise an exception if this handler shouldn't be exposed after all.
        pass


class HtmlWrapperHandler(WrapperHandler):
    global_type = None
    headers = [('Content-Type', 'text/html')]

    def check_exposure(self, request):
        if self.global_type:
            globals = u""
            for (key, value) in self._get_metadata(request):
                if key == "global":
                    globals = value
                    break

            if self.global_type not in parse_variants(globals):
                raise HTTPException(404, "This test cannot be loaded in %s mode" %
                                    self.global_type)

    def _meta_replacement(self, key, value):
        if key == "timeout":
            if value == "long":
                return '<meta name="timeout" content="long">'
        if key == "title":
            value = value.replace("&", "&amp;").replace("<", "&lt;")
            return '<title>%s</title>' % value
        return None

    def _script_replacement(self, key, value):
        if key == "script":
            attribute = value.replace("&", "&amp;").replace('"', "&quot;")
            return '<script src="%s"></script>' % attribute
        return None


class WorkersHandler(HtmlWrapperHandler):
    global_type = "dedicatedworker"
    path_replace = [(".any.worker.html", ".any.js", ".any.worker.js"),
                    (".worker.html", ".worker.js")]
    wrapper = """<!doctype html>
<meta charset=utf-8>
%(meta)s
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<div id=log></div>
<script>
fetch_tests_from_worker(new Worker("%(path)s%(query)s"));
</script>
"""


class WorkerModulesHandler(HtmlWrapperHandler):
    global_type = "dedicatedworker-module"
    path_replace = [(".any.worker-module.html", ".any.js", ".any.worker-module.js"),
                    (".worker.html", ".worker.js")]
    wrapper = """<!doctype html>
<meta charset=utf-8>
%(meta)s
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<div id=log></div>
<script>
fetch_tests_from_worker(new Worker("%(path)s%(query)s", { type: "module" }));
</script>
"""


class WindowHandler(HtmlWrapperHandler):
    path_replace = [(".window.html", ".window.js")]
    wrapper = """<!doctype html>
<meta charset=utf-8>
%(meta)s
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
%(script)s
<div id=log></div>
<script src="%(path)s"></script>
"""


class AnyHtmlHandler(HtmlWrapperHandler):
    global_type = "window"
    path_replace = [(".any.html", ".any.js")]
    wrapper = """<!doctype html>
<meta charset=utf-8>
%(meta)s
<script>
self.GLOBAL = {
  isWindow: function() { return true; },
  isWorker: function() { return false; },
};
</script>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
%(script)s
<div id=log></div>
<script src="%(path)s"></script>
"""


class SharedWorkersHandler(HtmlWrapperHandler):
    global_type = "sharedworker"
    path_replace = [(".any.sharedworker.html", ".any.js", ".any.worker.js")]
    wrapper = """<!doctype html>
<meta charset=utf-8>
%(meta)s
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<div id=log></div>
<script>
fetch_tests_from_worker(new SharedWorker("%(path)s%(query)s"));
</script>
"""


class SharedWorkerModulesHandler(HtmlWrapperHandler):
    global_type = "sharedworker-module"
    path_replace = [(".any.sharedworker-module.html", ".any.js", ".any.worker-module.js")]
    wrapper = """<!doctype html>
<meta charset=utf-8>
%(meta)s
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<div id=log></div>
<script>
fetch_tests_from_worker(new SharedWorker("%(path)s%(query)s", { type: "module" }));
</script>
"""


class ServiceWorkersHandler(HtmlWrapperHandler):
    global_type = "serviceworker"
    path_replace = [(".any.serviceworker.html", ".any.js", ".any.worker.js")]
    wrapper = """<!doctype html>
<meta charset=utf-8>
%(meta)s
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<div id=log></div>
<script>
(async function() {
  const scope = 'does/not/exist';
  let reg = await navigator.serviceWorker.getRegistration(scope);
  if (reg) await reg.unregister();
  reg = await navigator.serviceWorker.register("%(path)s%(query)s", {scope});
  fetch_tests_from_worker(reg.installing);
})();
</script>
"""


class ServiceWorkerModulesHandler(HtmlWrapperHandler):
    global_type = "serviceworker-module"
    path_replace = [(".any.serviceworker-module.html",
                     ".any.js", ".any.worker-module.js")]
    wrapper = """<!doctype html>
<meta charset=utf-8>
%(meta)s
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<div id=log></div>
<script>
(async function() {
  const scope = 'does/not/exist';
  let reg = await navigator.serviceWorker.getRegistration(scope);
  if (reg) await reg.unregister();
  reg = await navigator.serviceWorker.register(
    "%(path)s%(query)s",
    { scope, type: 'module' },
  );
  fetch_tests_from_worker(reg.installing);
})();
</script>
"""


class BaseWorkerHandler(WrapperHandler):
    headers = [('Content-Type', 'text/javascript')]

    def _meta_replacement(self, key, value):
        return None

    @abc.abstractmethod
    def _create_script_import(self, attribute):
        # Take attribute (a string URL to a JS script) and return JS source to import the script
        # into the worker.
        pass

    def _script_replacement(self, key, value):
        if key == "script":
            attribute = value.replace("\\", "\\\\").replace('"', '\\"')
            return self._create_script_import(attribute)
        if key == "title":
            value = value.replace("\\", "\\\\").replace('"', '\\"')
            return 'self.META_TITLE = "%s";' % value
        return None


class ClassicWorkerHandler(BaseWorkerHandler):
    path_replace = [(".any.worker.js", ".any.js")]
    wrapper = """%(meta)s
self.GLOBAL = {
  isWindow: function() { return false; },
  isWorker: function() { return true; },
};
importScripts("/resources/testharness.js");
%(script)s
importScripts("%(path)s");
done();
"""

    def _create_script_import(self, attribute):
        return 'importScripts("%s")' % attribute


class ModuleWorkerHandler(BaseWorkerHandler):
    path_replace = [(".any.worker-module.js", ".any.js")]
    wrapper = """%(meta)s
self.GLOBAL = {
  isWindow: function() { return false; },
  isWorker: function() { return true; },
};
import "/resources/testharness.js";
%(script)s
import "%(path)s";
done();
"""

    def _create_script_import(self, attribute):
        return 'import "%s";' % attribute


rewrites = [("GET", "/resources/WebIDLParser.js", "/resources/webidl2/lib/webidl2.js")]


class RoutesBuilder(object):
    def __init__(self):
        self.forbidden_override = [("GET", "/tools/runner/*", handlers.file_handler),
                                   ("POST", "/tools/runner/update_manifest.py",
                                    handlers.python_script_handler)]

        self.forbidden = [("*", "/_certs/*", handlers.ErrorHandler(404)),
                          ("*", "/tools/*", handlers.ErrorHandler(404)),
                          ("*", "{spec}/tools/*", handlers.ErrorHandler(404)),
                          ("*", "/results/", handlers.ErrorHandler(404))]

        self.extra = []

        self.mountpoint_routes = OrderedDict()

        self.add_mount_point("/", None)

    def get_routes(self):
        routes = self.forbidden_override + self.forbidden + self.extra
        # Using reversed here means that mount points that are added later
        # get higher priority. This makes sense since / is typically added
        # first.
        for item in reversed(self.mountpoint_routes.values()):
            routes.extend(item)
        return routes

    def add_handler(self, method, route, handler):
        self.extra.append((str(method), str(route), handler))

    def add_static(self, path, format_args, content_type, route, headers=None):
        if headers is None:
            headers = {}
        handler = handlers.StaticHandler(path, format_args, content_type, **headers)
        self.add_handler("GET", str(route), handler)

    def add_mount_point(self, url_base, path):
        url_base = "/%s/" % url_base.strip("/") if url_base != "/" else "/"

        self.mountpoint_routes[url_base] = []

        routes = [
            ("GET", "*.worker.html", WorkersHandler),
            ("GET", "*.worker-module.html", WorkerModulesHandler),
            ("GET", "*.window.html", WindowHandler),
            ("GET", "*.any.html", AnyHtmlHandler),
            ("GET", "*.any.sharedworker.html", SharedWorkersHandler),
            ("GET", "*.any.sharedworker-module.html", SharedWorkerModulesHandler),
            ("GET", "*.any.serviceworker.html", ServiceWorkersHandler),
            ("GET", "*.any.serviceworker-module.html", ServiceWorkerModulesHandler),
            ("GET", "*.any.worker.js", ClassicWorkerHandler),
            ("GET", "*.any.worker-module.js", ModuleWorkerHandler),
            ("GET", "*.asis", handlers.AsIsHandler),
            ("GET", "/.well-known/origin-policy", handlers.PythonScriptHandler),
            ("*", "*.py", handlers.PythonScriptHandler),
            ("GET", "*", handlers.FileHandler)
        ]

        for (method, suffix, handler_cls) in routes:
            self.mountpoint_routes[url_base].append(
                (method,
                 "%s%s" % (url_base if url_base != "/" else "", suffix),
                 handler_cls(base_path=path, url_base=url_base)))

    def add_file_mount_point(self, file_url, base_path):
        assert file_url.startswith("/")
        url_base = file_url[0:file_url.rfind("/") + 1]
        self.mountpoint_routes[file_url] = [("GET", file_url, handlers.FileHandler(base_path=base_path, url_base=url_base))]


def get_route_builder(aliases, config=None):
    builder = RoutesBuilder()
    for alias in aliases:
        url = alias["url-path"]
        directory = alias["local-dir"]
        if not url.startswith("/") or len(directory) == 0:
            logger.error("\"url-path\" value must start with '/'.")
            continue
        if url.endswith("/"):
            builder.add_mount_point(url, directory)
        else:
            builder.add_file_mount_point(url, directory)
    return builder


class ServerProc(object):
    def __init__(self, mp_context, scheme=None):
        self.proc = None
        self.daemon = None
        self.mp_context = mp_context
        self.stop = mp_context.Event()
        self.scheme = scheme

    def start(self, init_func, host, port, paths, routes, bind_address, config, **kwargs):
        self.proc = self.mp_context.Process(target=self.create_daemon,
                                            args=(init_func, host, port, paths, routes, bind_address,
                                                  config),
                                            name='%s on port %s' % (self.scheme, port),
                                            kwargs=kwargs)
        self.proc.daemon = True
        self.proc.start()

    def create_daemon(self, init_func, host, port, paths, routes, bind_address,
                      config, **kwargs):
        if sys.platform == "darwin":
            # on Darwin, NOFILE starts with a very low limit (256), so bump it up a little
            # by way of comparison, Debian starts with a limit of 1024, Windows 512
            import resource  # local, as it only exists on Unix-like systems
            maxfilesperproc = int(subprocess.check_output(
                ["sysctl", "-n", "kern.maxfilesperproc"]
            ).strip())
            soft, hard = resource.getrlimit(resource.RLIMIT_NOFILE)
            # 2048 is somewhat arbitrary, but gives us some headroom for wptrunner --parallel
            # note that it's expected that 2048 will be the min here
            new_soft = min(2048, maxfilesperproc, hard)
            if soft < new_soft:
                resource.setrlimit(resource.RLIMIT_NOFILE, (new_soft, hard))
        try:
            self.daemon = init_func(host, port, paths, routes, bind_address, config, **kwargs)
        except socket.error:
            logger.critical("Socket error on port %s" % port, file=sys.stderr)
            raise
        except Exception:
            logger.critical(traceback.format_exc())
            raise

        if self.daemon:
            try:
                self.daemon.start(block=False)
                try:
                    self.stop.wait()
                except KeyboardInterrupt:
                    pass
            except Exception:
                print(traceback.format_exc(), file=sys.stderr)
                raise

    def wait(self):
        self.stop.set()
        self.proc.join()

    def kill(self):
        self.stop.set()
        self.proc.terminate()
        self.proc.join()

    def is_alive(self):
        return self.proc.is_alive()


def check_subdomains(config, routes, mp_context):
    paths = config.paths
    bind_address = config.bind_address

    host = config.server_host
    port = get_port()
    logger.debug("Going to use port %d to check subdomains" % port)

    wrapper = ServerProc(mp_context)
    wrapper.start(start_http_server, host, port, paths, routes,
                  bind_address, config)

    url = "http://{}:{}/".format(host, port)
    connected = False
    for i in range(10):
        try:
            urllib.request.urlopen(url)
            connected = True
            break
        except urllib.error.URLError:
            time.sleep(1)

    if not connected:
        logger.critical("Failed to connect to test server "
                        "on {}. {}".format(url, EDIT_HOSTS_HELP))
        sys.exit(1)

    for domain in config.domains_set:
        if domain == host:
            continue

        try:
            urllib.request.urlopen("http://%s:%d/" % (domain, port))
        except Exception:
            logger.critical("Failed probing domain {}. {}".format(domain, EDIT_HOSTS_HELP))
            sys.exit(1)

    wrapper.wait()


def make_hosts_file(config, host):
    rv = []

    for domain in config.domains_set:
        rv.append("%s\t%s\n" % (host, domain))

    # Windows interpets the IP address 0.0.0.0 as non-existent, making it an
    # appropriate alias for non-existent hosts. However, UNIX-like systems
    # interpret the same address to mean any IP address, which is inappropraite
    # for this context. These systems do not reserve any value for this
    # purpose, so the inavailability of the domains must be taken for granted.
    #
    # https://github.com/web-platform-tests/wpt/issues/10560
    if platform.uname()[0] == "Windows":
        for not_domain in config.not_domains_set:
            rv.append("0.0.0.0\t%s\n" % not_domain)

    return "".join(rv)


def start_servers(host, ports, paths, routes, bind_address, config,
                  mp_context, **kwargs):
    servers = defaultdict(list)
    for scheme, ports in ports.items():
        assert len(ports) == {"http": 2, "https": 2}.get(scheme, 1)

        # If trying to start HTTP/2.0 server, check compatibility
        if scheme == 'h2' and not http2_compatible():
            logger.error('Cannot start HTTP/2.0 server as the environment is not compatible. ' +
                         'Requires Python 2.7.10+ or 3.6+ and OpenSSL 1.0.2+')
            continue

        for port in ports:
            if port is None:
                continue
            init_func = {"http": start_http_server,
                         "https": start_https_server,
                         "h2": start_http2_server,
                         "ws": start_ws_server,
                         "wss": start_wss_server,
                         "quic-transport": start_quic_transport_server}[scheme]

            server_proc = ServerProc(mp_context, scheme=scheme)
            server_proc.start(init_func, host, port, paths, routes, bind_address,
                              config, **kwargs)
            servers[scheme].append((port, server_proc))

    return servers


def startup_failed(log=True):
    # Log=False is a workaround for https://github.com/web-platform-tests/wpt/issues/22719
    if log:
        logger.critical(EDIT_HOSTS_HELP)
    else:
        print("CRITICAL %s" % EDIT_HOSTS_HELP, file=sys.stderr)
    sys.exit(1)


def start_http_server(host, port, paths, routes, bind_address, config, **kwargs):
    try:
        return wptserve.WebTestHttpd(host=host,
                                     port=port,
                                     doc_root=paths["doc_root"],
                                     routes=routes,
                                     rewrites=rewrites,
                                     bind_address=bind_address,
                                     config=config,
                                     use_ssl=False,
                                     key_file=None,
                                     certificate=None,
                                     latency=kwargs.get("latency"))
    except Exception:
        startup_failed()


def start_https_server(host, port, paths, routes, bind_address, config, **kwargs):
    try:
        return wptserve.WebTestHttpd(host=host,
                                     port=port,
                                     doc_root=paths["doc_root"],
                                     routes=routes,
                                     rewrites=rewrites,
                                     bind_address=bind_address,
                                     config=config,
                                     use_ssl=True,
                                     key_file=config.ssl_config["key_path"],
                                     certificate=config.ssl_config["cert_path"],
                                     encrypt_after_connect=config.ssl_config["encrypt_after_connect"],
                                     latency=kwargs.get("latency"))
    except Exception:
        startup_failed()


def start_http2_server(host, port, paths, routes, bind_address, config, **kwargs):
    try:
        return wptserve.WebTestHttpd(host=host,
                                     port=port,
                                     handler_cls=wptserve.Http2WebTestRequestHandler,
                                     doc_root=paths["doc_root"],
                                     ws_doc_root=paths["ws_doc_root"],
                                     routes=routes,
                                     rewrites=rewrites,
                                     bind_address=bind_address,
                                     config=config,
                                     use_ssl=True,
                                     key_file=config.ssl_config["key_path"],
                                     certificate=config.ssl_config["cert_path"],
                                     encrypt_after_connect=config.ssl_config["encrypt_after_connect"],
                                     latency=kwargs.get("latency"),
                                     http2=True)
    except Exception:
        startup_failed()


class WebSocketDaemon(object):
    def __init__(self, host, port, doc_root, handlers_root, bind_address, ssl_config):
        self.host = host
        cmd_args = ["-p", port,
                    "-d", doc_root,
                    "-w", handlers_root]

        if ssl_config is not None:
            cmd_args += ["--tls",
                         "--private-key", ssl_config["key_path"],
                         "--certificate", ssl_config["cert_path"]]

        if (bind_address):
            cmd_args = ["-H", host] + cmd_args
        opts, args = pywebsocket._parse_args_and_config(cmd_args)
        opts.cgi_directories = []
        opts.is_executable_method = None
        self.server = pywebsocket.WebSocketServer(opts)
        ports = [item[0].getsockname()[1] for item in self.server._sockets]
        if not ports:
            # TODO: Fix the logging configuration in WebSockets processes
            # see https://github.com/web-platform-tests/wpt/issues/22719
            print("Failed to start websocket server on port %s, "
                  "is something already using that port?" % port, file=sys.stderr)
            raise OSError()
        assert all(item == ports[0] for item in ports)
        self.port = ports[0]
        self.started = False
        self.server_thread = None

    def start(self, block=False):
        self.started = True
        if block:
            self.server.serve_forever()
        else:
            self.server_thread = threading.Thread(target=self.server.serve_forever)
            self.server_thread.setDaemon(True)  # don't hang on exit
            self.server_thread.start()

    def stop(self):
        """
        Stops the server.

        If the server is not running, this method has no effect.
        """
        if self.started:
            try:
                self.server.shutdown()
                self.server.server_close()
                self.server_thread.join()
                self.server_thread = None
            except AttributeError:
                pass
            self.started = False
        self.server = None


def release_mozlog_lock():
    try:
        from mozlog.structuredlog import StructuredLogger
        try:
            StructuredLogger._lock.release()
        except threading.ThreadError:
            pass
    except ImportError:
        pass


def start_ws_server(host, port, paths, routes, bind_address, config, **kwargs):
    # Ensure that when we start this in a new process we have the global lock
    # in the logging module unlocked
    importlib.reload(logging)
    release_mozlog_lock()
    try:
        return WebSocketDaemon(host,
                               str(port),
                               repo_root,
                               config.paths["ws_doc_root"],
                               bind_address,
                               ssl_config=None)
    except Exception:
        startup_failed(log=False)


def start_wss_server(host, port, paths, routes, bind_address, config, **kwargs):
    # Ensure that when we start this in a new process we have the global lock
    # in the logging module unlocked
    importlib.reload(logging)
    release_mozlog_lock()
    try:
        return WebSocketDaemon(host,
                               str(port),
                               repo_root,
                               config.paths["ws_doc_root"],
                               bind_address,
                               config.ssl_config)
    except Exception:
        startup_failed(log=False)


class QuicTransportDaemon(object):
    def __init__(self, host, port, handlers_path=None, private_key=None, certificate=None, log_level=None):
        args = ["python3", "wpt", "serve-quic-transport"]
        if host:
            args += ["--host", host]
        if port:
            args += ["--port", str(port)]
        if private_key:
            args += ["--private-key", private_key]
        if certificate:
            args += ["--certificate", certificate]
        if handlers_path:
            args += ["--handlers-path", handlers_path]
        if log_level == "debug":
            args += ["--verbose"]
        self.command = args
        self.proc = None

    def start(self, block=False):
        if block:
            subprocess.call(self.command)
        else:
            def handle_signal(*_):
                if self.proc:
                    try:
                        self.proc.terminate()
                    except OSError:
                        # It's fine if the child already exits.
                        pass
                    self.proc.wait()
                sys.exit(0)

            signal.signal(signal.SIGTERM, handle_signal)
            signal.signal(signal.SIGINT, handle_signal)

            self.proc = subprocess.Popen(self.command)
            # Give the server a second to start and then check.
            time.sleep(1)
            if self.proc.poll():
                sys.exit(1)


def start_quic_transport_server(host, port, paths, routes, bind_address, config, **kwargs):
    # Ensure that when we start this in a new process we have the global lock
    # in the logging module unlocked
    importlib.reload(logging)
    release_mozlog_lock()
    try:
        return QuicTransportDaemon(host,
                          port,
                          private_key=config.ssl_config["key_path"],
                          certificate=config.ssl_config["cert_path"],
                          log_level=config.log_level)
    except Exception:
        startup_failed(log=False)


def start(config, routes, mp_context, **kwargs):
    host = config["server_host"]
    ports = config.ports
    paths = config.paths
    bind_address = config["bind_address"]

    logger.debug("Using ports: %r" % ports)

    servers = start_servers(host, ports, paths, routes, bind_address, config, mp_context, **kwargs)

    return servers


def iter_procs(servers):
    for servers in servers.values():
        for port, server in servers:
            yield server.proc


def _make_subdomains_product(s, depth=2):
    return {u".".join(x) for x in chain(*(product(s, repeat=i) for i in range(1, depth+1)))}

def _make_origin_policy_subdomains(limit):
    return {u"op%d" % x for x in range(1,limit+1)}


_subdomains = {u"www",
               u"www1",
               u"www2",
               u"天気の良い日",
               u"élève"}

_not_subdomains = {u"nonexistent"}

_subdomains = _make_subdomains_product(_subdomains)

# Origin policy subdomains need to not be reused by any other tests, since origin policies have
# origin-wide impacts like installing a CSP or Feature Policy that could interfere with features
# under test.
# See https://github.com/web-platform-tests/rfcs/pull/44.
_subdomains |= _make_origin_policy_subdomains(99)

_not_subdomains = _make_subdomains_product(_not_subdomains)


class ConfigBuilder(config.ConfigBuilder):
    """serve config

    This subclasses wptserve.config.ConfigBuilder to add serve config options.
    """

    _default = {
        "browser_host": "web-platform.test",
        "alternate_hosts": {
            "alt": "not-web-platform.test"
        },
        "doc_root": repo_root,
        "ws_doc_root": os.path.join(repo_root, "websockets", "handlers"),
        "server_host": None,
        "ports": {
            "http": [8000, "auto"],
            "https": [8443, 8444],
            "ws": ["auto"],
            "wss": ["auto"],
        },
        "check_subdomains": True,
        "log_level": "info",
        "bind_address": True,
        "ssl": {
            "type": "pregenerated",
            "encrypt_after_connect": False,
            "openssl": {
                "openssl_binary": "openssl",
                "base_path": "_certs",
                "password": "web-platform-tests",
                "force_regenerate": False,
                "duration": 30,
                "base_conf_path": None
            },
            "pregenerated": {
                "host_key_path": os.path.join(repo_root, "tools", "certs", "web-platform.test.key"),
                "host_cert_path": os.path.join(repo_root, "tools", "certs", "web-platform.test.pem")
            },
            "none": {}
        },
        "aliases": []
    }

    computed_properties = ["ws_doc_root"] + config.ConfigBuilder.computed_properties

    def __init__(self, *args, **kwargs):
        if "subdomains" not in kwargs:
            kwargs["subdomains"] = _subdomains
        if "not_subdomains" not in kwargs:
            kwargs["not_subdomains"] = _not_subdomains
        super(ConfigBuilder, self).__init__(
            *args,
            **kwargs
        )
        with self as c:
            browser_host = c.get("browser_host")
            alternate_host = c.get("alternate_hosts", {}).get("alt")

            if not domains_are_distinct(browser_host, alternate_host):
                raise ValueError(
                    "Alternate host must be distinct from browser host"
                )

    def _get_ws_doc_root(self, data):
        if data["ws_doc_root"] is not None:
            return data["ws_doc_root"]
        else:
            return os.path.join(data["doc_root"], "websockets", "handlers")

    def ws_doc_root(self, v):
        self._ws_doc_root = v

    ws_doc_root = property(None, ws_doc_root)

    def _get_paths(self, data):
        rv = super(ConfigBuilder, self)._get_paths(data)
        rv["ws_doc_root"] = data["ws_doc_root"]
        return rv


def build_config(override_path=None, config_cls=ConfigBuilder, **kwargs):
    rv = config_cls()

    enable_http2 = kwargs.get("h2")
    if enable_http2 is None:
        enable_http2 = True
    if enable_http2:
        rv._default["ports"]["h2"] = [9000]

    if override_path and os.path.exists(override_path):
        with open(override_path) as f:
            override_obj = json.load(f)
        rv.update(override_obj)

    if kwargs.get("config_path"):
        other_path = os.path.abspath(os.path.expanduser(kwargs.get("config_path")))
        if os.path.exists(other_path):
            with open(other_path) as f:
                override_obj = json.load(f)
            rv.update(override_obj)
        else:
            raise ValueError("Config path %s does not exist" % other_path)

    if kwargs.get("verbose"):
        rv.log_level = "debug"

    overriding_path_args = [("doc_root", "Document root"),
                            ("ws_doc_root", "WebSockets document root")]
    for key, title in overriding_path_args:
        value = kwargs.get(key)
        if value is None:
            continue
        value = os.path.abspath(os.path.expanduser(value))
        if not os.path.exists(value):
            raise ValueError("%s path %s does not exist" % (title, value))
        setattr(rv, key, value)

    return rv


def get_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("--latency", type=int,
                        help="Artificial latency to add before sending http responses, in ms")
    parser.add_argument("--config", action="store", dest="config_path",
                        help="Path to external config file")
    parser.add_argument("--doc_root", action="store", dest="doc_root",
                        help="Path to document root. Overrides config.")
    parser.add_argument("--ws_doc_root", action="store", dest="ws_doc_root",
                        help="Path to WebSockets document root. Overrides config.")
    parser.add_argument("--alias_file", action="store", dest="alias_file",
                        help="File with entries for aliases/multiple doc roots. In form of `/ALIAS_NAME/, DOC_ROOT\\n`")
    parser.add_argument("--h2", action="store_true", dest="h2", default=None,
                        help=argparse.SUPPRESS)
    parser.add_argument("--no-h2", action="store_false", dest="h2", default=None,
                        help="Disable the HTTP/2.0 server")
    parser.add_argument("--quic-transport", action="store_true", help="Enable QUIC server for WebTransport")
    parser.add_argument("--exit-after-start", action="store_true", help="Exit after starting servers")
    parser.add_argument("--verbose", action="store_true", help="Enable verbose logging")
    parser.set_defaults(report=False)
    parser.set_defaults(is_wave=False)
    return parser


class MpContext(object):
    def __getattr__(self, name):
        return getattr(multiprocessing, name)


def run(config_cls=ConfigBuilder, route_builder=None, mp_context=None, **kwargs):
    received_signal = threading.Event()

    if mp_context is None:
        if hasattr(multiprocessing, "get_context"):
            mp_context = multiprocessing.get_context()
        else:
            mp_context = MpContext()

    with build_config(os.path.join(repo_root, "config.json"),
                      config_cls=config_cls,
                      **kwargs) as config:
        global logger
        logger = config.logger
        set_logger(logger)
        # Configure the root logger to cover third-party libraries.
        logging.getLogger().setLevel(config.log_level)

        def handle_signal(signum, frame):
            logger.debug("Received signal %s. Shutting down.", signum)
            received_signal.set()

        bind_address = config["bind_address"]

        if kwargs.get("alias_file"):
            with open(kwargs["alias_file"], 'r') as alias_file:
                for line in alias_file:
                    alias, doc_root = [x.strip() for x in line.split(',')]
                    config["aliases"].append({
                        'url-path': alias,
                        'local-dir': doc_root,
                    })

        if route_builder is None:
            route_builder = get_route_builder
        routes = route_builder(config.aliases, config).get_routes()

        if config["check_subdomains"]:
            check_subdomains(config, routes, mp_context)

        stash_address = None
        if bind_address:
            stash_address = (config.server_host, get_port(""))
            logger.debug("Going to use port %d for stash" % stash_address[1])

        with stash.StashServer(stash_address, authkey=str(uuid.uuid4())):
            servers = start(config, routes, mp_context, **kwargs)
            signal.signal(signal.SIGTERM, handle_signal)
            signal.signal(signal.SIGINT, handle_signal)

            while (all(subproc.is_alive() for subproc in iter_procs(servers)) and
                   not received_signal.is_set() and not kwargs["exit_after_start"]):
                for subproc in iter_procs(servers):
                    subproc.join(1)

            failed_subproc = 0
            for subproc in iter_procs(servers):
                if subproc.is_alive():
                    logger.info('Status of subprocess "%s": running' % subproc.name)
                else:
                    if subproc.exitcode == 0:
                        logger.info('Status of subprocess "%s": exited correctly' % subproc.name)
                    else:
                        logger.warning('Status of subprocess "%s": failed. Exit with non-zero status: %d' % (subproc.name, subproc.exitcode))
                        failed_subproc += 1
            return failed_subproc


def main():
    kwargs = vars(get_parser().parse_args())
    return run(**kwargs)
