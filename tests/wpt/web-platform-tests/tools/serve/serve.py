# -*- coding: utf-8 -*-

from __future__ import print_function

import abc
import argparse
import json
import os
import re
import socket
import sys
import threading
import time
import traceback
import urllib2
import uuid
from collections import defaultdict, OrderedDict
from multiprocessing import Process, Event

from ..localpaths import repo_root

import sslutils
from manifest.sourcefile import read_script_metadata, js_meta_re
from wptserve import server as wptserve, handlers
from wptserve import stash
from wptserve.logger import set_logger
from wptserve.handlers import filesystem_path, wrap_pipeline
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

    def __init__(self, base_path=None, url_base="/"):
        self.base_path = base_path
        self.url_base = url_base
        self.handler = handlers.handler(self.handle_request)

    def __call__(self, request, response):
        self.handler(request, response)

    def handle_request(self, request, response):
        path = self._get_path(request.url_parts.path, True)
        meta = "\n".join(self._get_meta(request))
        response.content = self.wrapper % {"meta": meta, "path": path}
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

    def _get_meta(self, request):
        """Get an iterator over strings to inject into the wrapper document
        based on //META comments in the associated js file.

        :param request: The Request being processed.
        """
        path = self._get_path(filesystem_path(self.base_path, request, self.url_base), False)
        with open(path, "rb") as f:
            for key, value in read_script_metadata(f, js_meta_re):
                replacement = self._meta_replacement(key, value)
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


class HtmlWrapperHandler(WrapperHandler):
    def _meta_replacement(self, key, value):
        if key == b"timeout":
            if value == b"long":
                return '<meta name="timeout" content="long">'
        if key == b"script":
            attribute = value.decode('utf-8').replace('"', "&quot;").replace(">", "&gt;")
            return '<script src="%s"></script>' % attribute
        return None


class WorkersHandler(HtmlWrapperHandler):
    path_replace = [(".any.worker.html", ".any.js", ".any.worker.js"),
                    (".worker.html", ".worker.js")]
    wrapper = """<!doctype html>
<meta charset=utf-8>
%(meta)s
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<div id=log></div>
<script>
fetch_tests_from_worker(new Worker("%(path)s"));
</script>
"""


class WindowHandler(HtmlWrapperHandler):
    path_replace = [(".window.html", ".window.js")]
    wrapper = """<!doctype html>
<meta charset=utf-8>
%(meta)s
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<div id=log></div>
<script src="%(path)s"></script>
"""


class AnyHtmlHandler(HtmlWrapperHandler):
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
<div id=log></div>
<script src="%(path)s"></script>
"""


class AnyWorkerHandler(WrapperHandler):
    path_replace = [(".any.worker.js", ".any.js")]
    wrapper = """%(meta)s
self.GLOBAL = {
  isWindow: function() { return false; },
  isWorker: function() { return true; },
};
importScripts("/resources/testharness.js");
importScripts("%(path)s");
done();
"""

    def _meta_replacement(self, key, value):
        if key == b"timeout":
            return None
        if key == b"script":
            attribute = value.decode('utf-8').replace("\\", "\\\\").replace('"', '\\"')
            return 'importScripts("%s")' % attribute
        return None


rewrites = [("GET", "/resources/WebIDLParser.js", "/resources/webidl2/lib/webidl2.js")]

subdomains = [u"www",
              u"www1",
              u"www2",
              u"天気の良い日",
              u"élève"]

class RoutesBuilder(object):
    def __init__(self):
        self.forbidden_override = [("GET", "/tools/runner/*", handlers.file_handler),
                                   ("POST", "/tools/runner/update_manifest.py",
                                    handlers.python_script_handler)]

        self.forbidden = [("*", "/_certs/*", handlers.ErrorHandler(404)),
                          ("*", "/tools/*", handlers.ErrorHandler(404)),
                          ("*", "{spec}/tools/*", handlers.ErrorHandler(404)),
                          ("*", "/serve.py", handlers.ErrorHandler(404))]

        self.static = []

        self.mountpoint_routes = OrderedDict()

        self.add_mount_point("/", None)

    def get_routes(self):
        routes = self.forbidden_override + self.forbidden + self.static
        # Using reversed here means that mount points that are added later
        # get higher priority. This makes sense since / is typically added
        # first.
        for item in reversed(self.mountpoint_routes.values()):
            routes.extend(item)
        return routes

    def add_static(self, path, format_args, content_type, route):
        handler = handlers.StaticHandler(path, format_args, content_type)
        self.static.append((b"GET", str(route), handler))

    def add_mount_point(self, url_base, path):
        url_base = "/%s/" % url_base.strip("/") if url_base != "/" else "/"

        self.mountpoint_routes[url_base] = []

        routes = [
            ("GET", "*.worker.html", WorkersHandler),
            ("GET", "*.window.html", WindowHandler),
            ("GET", "*.any.html", AnyHtmlHandler),
            ("GET", "*.any.worker.js", AnyWorkerHandler),
            ("GET", "*.asis", handlers.AsIsHandler),
            ("*", "*.py", handlers.PythonScriptHandler),
            ("GET", "*", handlers.FileHandler)
        ]

        for (method, suffix, handler_cls) in routes:
            self.mountpoint_routes[url_base].append(
                (method,
                 b"%s%s" % (str(url_base) if url_base != "/" else "", str(suffix)),
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


def setup_logger(level):
    import logging
    global logger
    logger = logging.getLogger("web-platform-tests")
    logging.basicConfig(level=getattr(logging, level.upper()))
    set_logger(logger)


def open_socket(port):
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    if port != 0:
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(('127.0.0.1', port))
    sock.listen(5)
    return sock

def bad_port(port):
    """
    Bad port as per https://fetch.spec.whatwg.org/#port-blocking
    """
    return port in [
        1,     # tcpmux
        7,     # echo
        9,     # discard
        11,    # systat
        13,    # daytime
        15,    # netstat
        17,    # qotd
        19,    # chargen
        20,    # ftp-data
        21,    # ftp
        22,    # ssh
        23,    # telnet
        25,    # smtp
        37,    # time
        42,    # name
        43,    # nicname
        53,    # domain
        77,    # priv-rjs
        79,    # finger
        87,    # ttylink
        95,    # supdup
        101,   # hostriame
        102,   # iso-tsap
        103,   # gppitnp
        104,   # acr-nema
        109,   # pop2
        110,   # pop3
        111,   # sunrpc
        113,   # auth
        115,   # sftp
        117,   # uucp-path
        119,   # nntp
        123,   # ntp
        135,   # loc-srv / epmap
        139,   # netbios
        143,   # imap2
        179,   # bgp
        389,   # ldap
        465,   # smtp+ssl
        512,   # print / exec
        513,   # login
        514,   # shell
        515,   # printer
        526,   # tempo
        530,   # courier
        531,   # chat
        532,   # netnews
        540,   # uucp
        556,   # remotefs
        563,   # nntp+ssl
        587,   # smtp
        601,   # syslog-conn
        636,   # ldap+ssl
        993,   # imap+ssl
        995,   # pop3+ssl
        2049,  # nfs
        3659,  # apple-sasl
        4045,  # lockd
        6000,  # x11
        6665,  # irc (alternate)
        6666,  # irc (alternate)
        6667,  # irc (default)
        6668,  # irc (alternate)
        6669,  # irc (alternate)
    ]

def get_port():
    port = 0
    while True:
        free_socket = open_socket(0)
        port = free_socket.getsockname()[1]
        free_socket.close()
        if not bad_port(port):
            break
    logger.debug("Going to use port %s" % port)
    return port


class ServerProc(object):
    def __init__(self):
        self.proc = None
        self.daemon = None
        self.stop = Event()

    def start(self, init_func, host, port, paths, routes, bind_hostname, external_config,
              ssl_config, **kwargs):
        self.proc = Process(target=self.create_daemon,
                            args=(init_func, host, port, paths, routes, bind_hostname,
                                  external_config, ssl_config),
                            kwargs=kwargs)
        self.proc.daemon = True
        self.proc.start()

    def create_daemon(self, init_func, host, port, paths, routes, bind_hostname,
                      external_config, ssl_config, **kwargs):
        try:
            self.daemon = init_func(host, port, paths, routes, bind_hostname, external_config,
                                    ssl_config, **kwargs)
        except socket.error:
            print("Socket error on port %s" % port, file=sys.stderr)
            raise
        except:
            print(traceback.format_exc(), file=sys.stderr)
            raise

        if self.daemon:
            try:
                self.daemon.start(block=False)
                try:
                    self.stop.wait()
                except KeyboardInterrupt:
                    pass
            except:
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


def check_subdomains(host, paths, bind_hostname, ssl_config, aliases):
    port = get_port()
    subdomains = get_subdomains(host)

    wrapper = ServerProc()
    wrapper.start(start_http_server, host, port, paths, build_routes(aliases), bind_hostname,
                  None, ssl_config)

    connected = False
    for i in range(10):
        try:
            urllib2.urlopen("http://%s:%d/" % (host, port))
            connected = True
            break
        except urllib2.URLError:
            time.sleep(1)

    if not connected:
        logger.critical("Failed to connect to test server on http://%s:%s You may need to edit /etc/hosts or similar" % (host, port))
        sys.exit(1)

    for subdomain, (punycode, host) in subdomains.iteritems():
        domain = "%s.%s" % (punycode, host)
        try:
            urllib2.urlopen("http://%s:%d/" % (domain, port))
        except Exception as e:
            logger.critical("Failed probing domain %s. You may need to edit /etc/hosts or similar." % domain)
            sys.exit(1)

    wrapper.wait()


def get_subdomains(host):
    #This assumes that the tld is ascii-only or already in punycode
    return {subdomain: (subdomain.encode("idna"), host)
            for subdomain in subdomains}


def start_servers(host, ports, paths, routes, bind_hostname, external_config, ssl_config,
                  **kwargs):
    servers = defaultdict(list)
    for scheme, ports in ports.iteritems():
        assert len(ports) == {"http":2}.get(scheme, 1)

        for port in ports:
            if port is None:
                continue
            init_func = {"http":start_http_server,
                         "https":start_https_server,
                         "ws":start_ws_server,
                         "wss":start_wss_server}[scheme]

            server_proc = ServerProc()
            server_proc.start(init_func, host, port, paths, routes, bind_hostname,
                              external_config, ssl_config, **kwargs)
            servers[scheme].append((port, server_proc))

    return servers


def start_http_server(host, port, paths, routes, bind_hostname, external_config, ssl_config,
                      **kwargs):
    return wptserve.WebTestHttpd(host=host,
                                 port=port,
                                 doc_root=paths["doc_root"],
                                 routes=routes,
                                 rewrites=rewrites,
                                 bind_hostname=bind_hostname,
                                 config=external_config,
                                 use_ssl=False,
                                 key_file=None,
                                 certificate=None,
                                 latency=kwargs.get("latency"))


def start_https_server(host, port, paths, routes, bind_hostname, external_config, ssl_config,
                       **kwargs):
    return wptserve.WebTestHttpd(host=host,
                                 port=port,
                                 doc_root=paths["doc_root"],
                                 routes=routes,
                                 rewrites=rewrites,
                                 bind_hostname=bind_hostname,
                                 config=external_config,
                                 use_ssl=True,
                                 key_file=ssl_config["key_path"],
                                 certificate=ssl_config["cert_path"],
                                 encrypt_after_connect=ssl_config["encrypt_after_connect"],
                                 latency=kwargs.get("latency"))


class WebSocketDaemon(object):
    def __init__(self, host, port, doc_root, handlers_root, log_level, bind_hostname,
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

        if (bind_hostname):
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


def start_ws_server(host, port, paths, routes, bind_hostname, external_config, ssl_config,
                    **kwargs):
    return WebSocketDaemon(host,
                           str(port),
                           repo_root,
                           paths["ws_doc_root"],
                           "debug",
                           bind_hostname,
                           ssl_config = None)


def start_wss_server(host, port, paths, routes, bind_hostname, external_config, ssl_config,
                     **kwargs):
    return WebSocketDaemon(host,
                           str(port),
                           repo_root,
                           paths["ws_doc_root"],
                           "debug",
                           bind_hostname,
                           ssl_config)


def get_ports(config, ssl_environment):
    rv = defaultdict(list)
    for scheme, ports in config["ports"].iteritems():
        for i, port in enumerate(ports):
            if scheme in ["wss", "https"] and not ssl_environment.ssl_enabled:
                port = None
            if port == "auto":
                port = get_port()
            else:
                port = port
            rv[scheme].append(port)
    return rv



def normalise_config(config, ports):
    host = config["external_host"] if config["external_host"] else config["host"]
    domains = get_subdomains(host)
    ports_ = {}
    for scheme, ports_used in ports.iteritems():
        ports_[scheme] = ports_used

    for key, value in domains.iteritems():
        domains[key] = ".".join(value)

    domains[""] = host

    ports_ = {}
    for scheme, ports_used in ports.iteritems():
        ports_[scheme] = ports_used

    return {"host": host,
            "domains": domains,
            "ports": ports_}


def get_ssl_config(config, external_domains, ssl_environment):
    key_path, cert_path = ssl_environment.host_cert_path(external_domains)
    return {"key_path": key_path,
            "cert_path": cert_path,
            "encrypt_after_connect": config["ssl"]["encrypt_after_connect"]}

def start(config, ssl_environment, routes, **kwargs):
    host = config["host"]
    domains = get_subdomains(host)
    ports = get_ports(config, ssl_environment)
    bind_hostname = config["bind_hostname"]

    paths = {"doc_root": config["doc_root"],
             "ws_doc_root": config["ws_doc_root"]}

    external_config = normalise_config(config, ports)

    ssl_config = get_ssl_config(config, external_config["domains"].values(), ssl_environment)

    if config["check_subdomains"]:
        check_subdomains(host, paths, bind_hostname, ssl_config, config["aliases"])

    servers = start_servers(host, ports, paths, routes, bind_hostname, external_config,
                            ssl_config, **kwargs)

    return external_config, servers


def iter_procs(servers):
    for servers in servers.values():
        for port, server in servers:
            yield server.proc


def value_set(config, key):
    return key in config and config[key] is not None


def get_value_or_default(config, key, default=None):
    return config[key] if value_set(config, key) else default


def set_computed_defaults(config):
    if not value_set(config, "doc_root"):
        config["doc_root"] = repo_root

    if not value_set(config, "ws_doc_root"):
        root = get_value_or_default(config, "doc_root", default=repo_root)
        config["ws_doc_root"] = os.path.join(root, "websockets", "handlers")

    if not value_set(config, "aliases"):
        config["aliases"] = []


def merge_json(base_obj, override_obj):
    rv = {}
    for key, value in base_obj.iteritems():
        if key not in override_obj:
            rv[key] = value
        else:
            if isinstance(value, dict):
                rv[key] = merge_json(value, override_obj[key])
            else:
                rv[key] = override_obj[key]
    return rv


def get_ssl_environment(config):
    implementation_type = config["ssl"]["type"]
    cls = sslutils.environments[implementation_type]
    try:
        kwargs = config["ssl"][implementation_type].copy()
    except KeyError:
        raise ValueError("%s is not a vaid ssl type." % implementation_type)
    return cls(logger, **kwargs)


def load_config(default_path, override_path=None, **kwargs):
    if os.path.exists(default_path):
        with open(default_path) as f:
            base_obj = json.load(f)
    else:
        raise ValueError("Config path %s does not exist" % default_path)

    if os.path.exists(override_path):
        with open(override_path) as f:
            override_obj = json.load(f)
    else:
        override_obj = {}
    rv = merge_json(base_obj, override_obj)

    if kwargs.get("config_path"):
        other_path = os.path.abspath(os.path.expanduser(kwargs.get("config_path")))
        if os.path.exists(other_path):
            base_obj = rv
            with open(other_path) as f:
                override_obj = json.load(f)
            rv = merge_json(base_obj, override_obj)
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
        rv[key] = value

    set_computed_defaults(rv)
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
    return parser


def main():
    kwargs = vars(get_parser().parse_args())
    config = load_config("config.default.json",
                         "config.json",
                         **kwargs)

    setup_logger(config["log_level"])

    stash_address = None
    if config["bind_hostname"]:
        stash_address = (config["host"], get_port())

    with stash.StashServer(stash_address, authkey=str(uuid.uuid4())):
        with get_ssl_environment(config) as ssl_env:
            config_, servers = start(config, ssl_env, build_routes(config["aliases"]), **kwargs)

            try:
                while any(item.is_alive() for item in iter_procs(servers)):
                    for item in iter_procs(servers):
                        item.join(1)
            except KeyboardInterrupt:
                logger.info("Shutting down")
