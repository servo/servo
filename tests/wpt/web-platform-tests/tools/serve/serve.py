# -*- coding: utf-8 -*-

from __future__ import print_function

import abc
import argparse
import json
import logging
import os
import platform
import socket
import sys
import threading
import time
import traceback
from six.moves import urllib
import uuid
from collections import defaultdict, OrderedDict
from itertools import chain, product
from multiprocessing import Process, Event

from localpaths import repo_root
from six.moves import reload_module

from manifest.sourcefile import read_script_metadata, js_meta_re, parse_variants
from wptserve import server as wptserve, handlers
from wptserve import stash
from wptserve import config
from wptserve.logger import set_logger
from wptserve.handlers import filesystem_path, wrap_pipeline
from wptserve.utils import get_port, HTTPException, http2_compatible
from mod_pywebsocket import standalone as pywebsocket


def replace_end(s, old, new):
    """
    Given a string `s` that ends with `old`, replace that occurrence of `old`
    with `new`.
    """
    assert s.endswith(old)
    return s[:-len(old)] + new


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
        for header_name, header_value in self.headers:
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

    def _get_metadata(self, request):
        """Get an iterator over script metadata based on // META comments in the
        associated js file.

        :param request: The Request being processed.
        """
        path = self._get_path(filesystem_path(self.base_path, request, self.url_base), False)
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
            globals = b""
            for (key, value) in self._get_metadata(request):
                if key == b"global":
                    globals = value
                    break

            if self.global_type not in parse_variants(globals):
                raise HTTPException(404, "This test cannot be loaded in %s mode" %
                                    self.global_type)

    def _meta_replacement(self, key, value):
        if key == b"timeout":
            if value == b"long":
                return '<meta name="timeout" content="long">'
        if key == b"title":
            value = value.decode('utf-8').replace("&", "&amp;").replace("<", "&lt;")
            return '<title>%s</title>' % value
        return None

    def _script_replacement(self, key, value):
        if key == b"script":
            attribute = value.decode('utf-8').replace("&", "&amp;").replace('"', "&quot;")
            return '<script src="%s"></script>' % attribute
        return None


class WorkersHandler(HtmlWrapperHandler):
    global_type = b"dedicatedworker"
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
    global_type = b"window"
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
    global_type = b"sharedworker"
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


class ServiceWorkersHandler(HtmlWrapperHandler):
    global_type = b"serviceworker"
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


class AnyWorkerHandler(WrapperHandler):
    headers = [('Content-Type', 'text/javascript')]
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

    def _meta_replacement(self, key, value):
        return None

    def _script_replacement(self, key, value):
        if key == b"script":
            attribute = value.decode('utf-8').replace("\\", "\\\\").replace('"', '\\"')
            return 'importScripts("%s")' % attribute
        if key == b"title":
            value = value.decode('utf-8').replace("\\", "\\\\").replace('"', '\\"')
            return 'self.META_TITLE = "%s";' % value
        return None


rewrites = [("GET", "/resources/WebIDLParser.js", "/resources/webidl2/lib/webidl2.js")]

class RoutesBuilder(object):
    def __init__(self):
        self.forbidden_override = [("GET", "/tools/runner/*", handlers.file_handler),
                                   ("POST", "/tools/runner/update_manifest.py",
                                    handlers.python_script_handler)]

        self.forbidden = [("*", "/_certs/*", handlers.ErrorHandler(404)),
                          ("*", "/tools/*", handlers.ErrorHandler(404)),
                          ("*", "{spec}/tools/*", handlers.ErrorHandler(404)),
                          ("*", "/serve.py", handlers.ErrorHandler(404))]

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
        self.add_handler(b"GET", str(route), handler)

    def add_mount_point(self, url_base, path):
        url_base = "/%s/" % url_base.strip("/") if url_base != "/" else "/"

        self.mountpoint_routes[url_base] = []

        routes = [
            ("GET", "*.worker.html", WorkersHandler),
            ("GET", "*.window.html", WindowHandler),
            ("GET", "*.any.html", AnyHtmlHandler),
            ("GET", "*.any.sharedworker.html", SharedWorkersHandler),
            ("GET", "*.any.serviceworker.html", ServiceWorkersHandler),
            ("GET", "*.any.worker.js", AnyWorkerHandler),
            ("GET", "*.asis", handlers.AsIsHandler),
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


def build_routes(aliases):
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
    return builder.get_routes()


class ServerProc(object):
    def __init__(self, scheme=None):
        self.proc = None
        self.daemon = None
        self.stop = Event()
        self.scheme = scheme

    def start(self, init_func, host, port, paths, routes, bind_address, config, **kwargs):
        self.proc = Process(target=self.create_daemon,
                            args=(init_func, host, port, paths, routes, bind_address,
                                  config),
                            name='%s on port %s' % (self.scheme, port),
                            kwargs=kwargs)
        self.proc.daemon = True
        self.proc.start()

    def create_daemon(self, init_func, host, port, paths, routes, bind_address,
                      config, **kwargs):
        try:
            self.daemon = init_func(host, port, paths, routes, bind_address, config, **kwargs)
        except socket.error:
            print("Socket error on port %s" % port, file=sys.stderr)
            raise
        except Exception:
            print(traceback.format_exc(), file=sys.stderr)
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


def check_subdomains(config):
    paths = config.paths
    bind_address = config.bind_address
    aliases = config.aliases

    host = config.server_host
    port = get_port()
    logger.debug("Going to use port %d to check subdomains" % port)

    wrapper = ServerProc()
    wrapper.start(start_http_server, host, port, paths, build_routes(aliases),
                  bind_address, config)

    connected = False
    for i in range(10):
        try:
            urllib.request.urlopen("http://%s:%d/" % (host, port))
            connected = True
            break
        except urllib.error.URLError:
            time.sleep(1)

    if not connected:
        logger.critical("Failed to connect to test server on http://%s:%s. "
                        "You may need to edit /etc/hosts or similar, see README.md." % (host, port))
        sys.exit(1)

    for domain in config.domains_set:
        if domain == host:
            continue

        try:
            urllib.request.urlopen("http://%s:%d/" % (domain, port))
        except Exception:
            logger.critical("Failed probing domain %s. "
                            "You may need to edit /etc/hosts or similar, see README.md." % domain)
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


def start_servers(host, ports, paths, routes, bind_address, config, **kwargs):
    servers = defaultdict(list)
    for scheme, ports in ports.items():
        assert len(ports) == {"http":2}.get(scheme, 1)

        # If trying to start HTTP/2.0 server, check compatibility
        if scheme == 'http2' and not http2_compatible():
            logger.error('Cannot start HTTP/2.0 server as the environment is not compatible. ' +
                         'Requires Python 2.7.10+ (< 3.0) and OpenSSL 1.0.2+')
            continue

        for port in ports:
            if port is None:
                continue
            init_func = {"http":start_http_server,
                         "https":start_https_server,
                         "http2":start_http2_server,
                         "ws":start_ws_server,
                         "wss":start_wss_server}[scheme]

            server_proc = ServerProc(scheme=scheme)
            server_proc.start(init_func, host, port, paths, routes, bind_address,
                              config, **kwargs)
            servers[scheme].append((port, server_proc))

    return servers


def start_http_server(host, port, paths, routes, bind_address, config, **kwargs):
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


def start_https_server(host, port, paths, routes, bind_address, config, **kwargs):
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


def start_http2_server(host, port, paths, routes, bind_address, config, **kwargs):
    return wptserve.WebTestHttpd(host=host,
                                 port=port,
                                 handler_cls=wptserve.Http2WebTestRequestHandler,
                                 doc_root=paths["doc_root"],
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
class WebSocketDaemon(object):
    def __init__(self, host, port, doc_root, handlers_root, log_level, bind_address,
                 ssl_config):
        self.host = host
        cmd_args = ["-p", port,
                    "-d", doc_root,
                    "-w", handlers_root,
                    "--log-level", log_level]

        if ssl_config is not None:
            # This is usually done through pywebsocket.main, however we're
            # working around that to get the server instance and manually
            # setup the wss server.
            if pywebsocket._import_ssl():
                tls_module = pywebsocket._TLS_BY_STANDARD_MODULE
            elif pywebsocket._import_pyopenssl():
                tls_module = pywebsocket._TLS_BY_PYOPENSSL
            else:
                print("No SSL module available")
                sys.exit(1)

            cmd_args += ["--tls",
                         "--private-key", ssl_config["key_path"],
                         "--certificate", ssl_config["cert_path"],
                         "--tls-module", tls_module]

        if (bind_address):
            cmd_args = ["-H", host] + cmd_args
        opts, args = pywebsocket._parse_args_and_config(cmd_args)
        opts.cgi_directories = []
        opts.is_executable_method = None
        self.server = pywebsocket.WebSocketServer(opts)
        ports = [item[0].getsockname()[1] for item in self.server._sockets]
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
    reload_module(logging)
    release_mozlog_lock()
    return WebSocketDaemon(host,
                           str(port),
                           repo_root,
                           config.paths["ws_doc_root"],
                           "debug",
                           bind_address,
                           ssl_config = None)


def start_wss_server(host, port, paths, routes, bind_address, config, **kwargs):
    # Ensure that when we start this in a new process we have the global lock
    # in the logging module unlocked
    reload_module(logging)
    release_mozlog_lock()
    return WebSocketDaemon(host,
                           str(port),
                           repo_root,
                           config.paths["ws_doc_root"],
                           "debug",
                           bind_address,
                           config.ssl_config)


def start(config, routes, **kwargs):
    host = config["server_host"]
    ports = config.ports
    paths = config.paths
    bind_address = config["bind_address"]

    logger.debug("Using ports: %r" % ports)

    servers = start_servers(host, ports, paths, routes, bind_address, config, **kwargs)

    return servers


def iter_procs(servers):
    for servers in servers.values():
        for port, server in servers:
            yield server.proc


def build_config(override_path=None, **kwargs):
    rv = ConfigBuilder()

    if kwargs.get("h2"):
        rv._default["ports"]["http2"] = [9000]

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

def _make_subdomains_product(s, depth=2):
    return {u".".join(x) for x in chain(*(product(s, repeat=i) for i in range(1, depth+1)))}

_subdomains = {u"www",
               u"www1",
               u"www2",
               u"天気の良い日",
               u"élève"}

_not_subdomains = {u"nonexistent"}

_subdomains = _make_subdomains_product(_subdomains)

_not_subdomains = _make_subdomains_product(_not_subdomains)


class ConfigBuilder(config.ConfigBuilder):
    """serve config

    this subclasses wptserve.config.ConfigBuilder to add serve config options"""

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
            "https": [8443],
            "ws": ["auto"],
            "wss": ["auto"],
        },
        "check_subdomains": True,
        "log_level": "debug",
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
    parser.add_argument("--h2", action="store_true", dest="h2",
                        help="Flag for enabling the HTTP/2.0 server")
    parser.set_defaults(h2=False)
    return parser


def run(**kwargs):
    with build_config(os.path.join(repo_root, "config.json"),
                      **kwargs) as config:
        global logger
        logger = config.logger
        set_logger(logger)

        bind_address = config["bind_address"]

        if kwargs.get("alias_file"):
            with open(kwargs["alias_file"], 'r') as alias_file:
                for line in alias_file:
                    alias, doc_root = [x.strip() for x in line.split(',')]
                    config["aliases"].append({
                        'url-path': alias,
                        'local-dir': doc_root,
                    })

        if config["check_subdomains"]:
            check_subdomains(config)

        stash_address = None
        if bind_address:
            stash_address = (config.server_host, get_port(""))
            logger.debug("Going to use port %d for stash" % stash_address[1])

        with stash.StashServer(stash_address, authkey=str(uuid.uuid4())):
            servers = start(config, build_routes(config["aliases"]), **kwargs)

            try:
                while all(item.is_alive() for item in iter_procs(servers)):
                    for item in iter_procs(servers):
                        item.join(1)
                exited = [item for item in iter_procs(servers) if not item.is_alive()]
                subject = "subprocess" if len(exited) == 1 else "subprocesses"

                logger.info("%s %s exited:" % (len(exited), subject))

                for item in iter_procs(servers):
                    logger.info("Status of %s:\t%s" % (item.name, "running" if item.is_alive() else "not running"))
            except KeyboardInterrupt:
                logger.info("Shutting down")


def main():
    kwargs = vars(get_parser().parse_args())
    return run(**kwargs)
