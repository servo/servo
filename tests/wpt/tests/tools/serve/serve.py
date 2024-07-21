# mypy: allow-untyped-defs

import abc
import argparse
import importlib
import json
import logging
import multiprocessing
import os
import platform
import subprocess
import sys
import threading
import time
import traceback
import urllib
import uuid
from collections import defaultdict, OrderedDict
from io import IOBase
from itertools import chain, product
from html5lib import html5parser
from typing import ClassVar, List, Optional, Set, Tuple

from localpaths import repo_root  # type: ignore

from manifest.sourcefile import read_script_metadata, js_meta_re, parse_variants  # type: ignore
from wptserve import server as wptserve, handlers
from wptserve import stash
from wptserve import config
from wptserve.handlers import filesystem_path, wrap_pipeline
from wptserve.response import ResponseHeaders
from wptserve.utils import get_port, HTTPException, http2_compatible
from pywebsocket3 import standalone as pywebsocket


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


def inject_script(html, script_tag):
    # Tokenize and find the position of the first content (e.g. after the
    # doctype, html, and head opening tags if present but before any other tags).
    token_types = html5parser.tokenTypes
    after_tags = {"html", "head"}
    before_tokens = {token_types["EndTag"], token_types["EmptyTag"],
                     token_types["Characters"]}
    error_tokens = {token_types["ParseError"]}

    tokenizer = html5parser._tokenizer.HTMLTokenizer(html)
    stream = tokenizer.stream
    offset = 0
    error = False
    for item in tokenizer:
        if item["type"] == token_types["StartTag"]:
            if not item["name"].lower() in after_tags:
                break
        elif item["type"] in before_tokens:
            break
        elif item["type"] in error_tokens:
            error = True
            break
        offset = stream.chunkOffset
    else:
        error = True

    if not error and stream.prevNumCols or stream.prevNumLines:
        # We're outside the first chunk, so we don't know what to do
        error = True

    if error:
        return html
    else:
        return html[:offset] + script_tag + html[offset:]


class WrapperHandler:

    __meta__ = abc.ABCMeta

    headers: ClassVar[List[Tuple[str, str]]] = []

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
                yield from read_script_metadata(f, js_meta_re)
        except OSError:
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
    global_type: ClassVar[Optional[str]] = None
    headers = [('Content-Type', 'text/html')]

    def check_exposure(self, request):
        if self.global_type is not None:
            global_variants = ""
            for (key, value) in self._get_metadata(request):
                if key == "global":
                    global_variants = value
                    break

            if self.global_type not in parse_variants(global_variants):
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


class HtmlScriptInjectorHandlerWrapper:
    def __init__(self, inject="", wrap=None):
        self.inject = inject
        self.wrap = wrap

    def __call__(self, request, response):
        self.wrap(request, response)
        # If the response content type isn't html, don't modify it.
        if not isinstance(response.headers, ResponseHeaders) or response.headers.get("Content-Type")[0] != b"text/html":
            return response

        # Skip injection on custom streaming responses.
        if not isinstance(response.content, (bytes, str, IOBase)) and not hasattr(response, "read"):
            return response

        response.content = inject_script(
            b"".join(response.iter_content(read_file=True)),
            b"<script>\n" +
            self.inject + b"\n" +
            (b"// Remove the injected script tag from the DOM.\n"
            b"document.currentScript.remove();\n"
            b"</script>\n"))
        return response


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


class WindowModulesHandler(HtmlWrapperHandler):
    global_type = "window-module"
    path_replace = [(".any.window-module.html", ".any.js")]
    wrapper = """<!doctype html>
<meta charset=utf-8>
%(meta)s
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
%(script)s
<div id=log></div>
<script type=module src="%(path)s"></script>
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
  isShadowRealm: function() { return false; },
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

class ShadowRealmHandler(HtmlWrapperHandler):
    global_type = "shadowrealm"
    path_replace = [(".any.shadowrealm.html", ".any.js")]

    wrapper = """<!doctype html>
<meta charset=utf-8>
%(meta)s
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script>
(async function() {
  const r = new ShadowRealm();
  r.evaluate("globalThis.self = globalThis; undefined;");
  r.evaluate(`func => {
    globalThis.fetch_json = (resource) => {
      const thenMethod = func(resource);
      return new Promise((resolve, reject) => thenMethod((s) => resolve(JSON.parse(s)), reject));
    };
  }`)((resource) => function (resolve, reject) {
    fetch(resource).then(res => res.text(), String).then(resolve, reject);
  });
  r.evaluate(`s => {
    globalThis.location = { search: s };
  }`)(location.search);
  await new Promise(r.evaluate(`
    (resolve, reject) => {
      (async () => {
        globalThis.self.GLOBAL = {
          isWindow: function() { return false; },
          isWorker: function() { return false; },
          isShadowRealm: function() { return true; },
        };
        await import("/resources/testharness.js");
        %(script)s
        await import("%(path)s");
      })().then(resolve, (e) => reject(e.toString()));
    }
  `));

  await fetch_tests_from_shadow_realm(r);
  done();
})();
</script>
"""

    def _script_replacement(self, key, value):
        if key == "script":
            return 'await import("%s");' % value
        return None


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
  isShadowRealm: function() { return false; },
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
  isShadowRealm: function() { return false; },
};
import "/resources/testharness.js";
%(script)s
import "%(path)s";
done();
"""

    def _create_script_import(self, attribute):
        return 'import "%s";' % attribute


rewrites = [("GET", "/resources/WebIDLParser.js", "/resources/webidl2/lib/webidl2.js")]


class RoutesBuilder:
    def __init__(self, inject_script = None):
        self.forbidden_override = [("GET", "/tools/runner/*", handlers.file_handler),
                                   ("POST", "/tools/runner/update_manifest.py",
                                    handlers.python_script_handler)]

        self.forbidden = [("*", "/_certs/*", handlers.ErrorHandler(404)),
                          ("*", "/tools/*", handlers.ErrorHandler(404)),
                          ("*", "{spec}/tools/*", handlers.ErrorHandler(404)),
                          ("*", "/results/", handlers.ErrorHandler(404))]

        self.extra = []
        self.inject_script_data = None
        if inject_script is not None:
            with open(inject_script, 'rb') as f:
                self.inject_script_data = f.read()

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
            ("GET", "*.any.shadowrealm.html", ShadowRealmHandler),
            ("GET", "*.any.window-module.html", WindowModulesHandler),
            ("GET", "*.any.worker.js", ClassicWorkerHandler),
            ("GET", "*.any.worker-module.js", ModuleWorkerHandler),
            ("GET", "*.asis", handlers.AsIsHandler),
            ("*", "/.well-known/attribution-reporting/report-event-attribution", handlers.PythonScriptHandler),
            ("*", "/.well-known/attribution-reporting/debug/report-event-attribution", handlers.PythonScriptHandler),
            ("*", "/.well-known/attribution-reporting/report-aggregate-attribution", handlers.PythonScriptHandler),
            ("*", "/.well-known/attribution-reporting/debug/report-aggregate-attribution", handlers.PythonScriptHandler),
            ("*", "/.well-known/attribution-reporting/debug/report-aggregate-debug", handlers.PythonScriptHandler),
            ("*", "/.well-known/attribution-reporting/debug/verbose", handlers.PythonScriptHandler),
            ("GET", "/.well-known/interest-group/permissions/", handlers.PythonScriptHandler),
            ("*", "/.well-known/private-aggregation/*", handlers.PythonScriptHandler),
            ("*", "/.well-known/web-identity", handlers.PythonScriptHandler),
            ("*", "*.py", handlers.PythonScriptHandler),
            ("GET", "*", handlers.FileHandler)
        ]

        for (method, suffix, handler_cls) in routes:
            handler = handler_cls(base_path=path, url_base=url_base)
            if self.inject_script_data is not None:
                handler = HtmlScriptInjectorHandlerWrapper(inject=self.inject_script_data, wrap=handler)

            self.mountpoint_routes[url_base].append(
                (method,
                 "%s%s" % (url_base if url_base != "/" else "", suffix),
                 handler))

    def add_file_mount_point(self, file_url, base_path):
        assert file_url.startswith("/")
        url_base = file_url[0:file_url.rfind("/") + 1]
        self.mountpoint_routes[file_url] = [("GET", file_url, handlers.FileHandler(base_path=base_path, url_base=url_base))]


def get_route_builder(logger, aliases, config):
    builder = RoutesBuilder(config.inject_script)
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


class ServerProc:
    def __init__(self, mp_context, scheme=None):
        self.proc = None
        self.daemon = None
        self.mp_context = mp_context
        self.stop_flag = mp_context.Event()
        self.scheme = scheme

    def start(self, init_func, host, port, paths, routes, bind_address, config, log_handlers, **kwargs):
        self.proc = self.mp_context.Process(target=self.create_daemon,
                                            args=(init_func, host, port, paths, routes, bind_address,
                                                  config, log_handlers, dict(**os.environ)),
                                            name='%s on port %s' % (self.scheme, port),
                                            kwargs=kwargs)
        self.proc.daemon = True
        self.proc.start()

    def create_daemon(self, init_func, host, port, paths, routes, bind_address,
                      config, log_handlers, env, **kwargs):
        # Ensure that when we start this in a new process we have the global lock
        # in the logging module unlocked
        importlib.reload(logging)
        os.environ = env
        logger = get_logger(config.logging["level"], log_handlers)

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
            self.daemon = init_func(logger, host, port, paths, routes, bind_address, config, **kwargs)
        except OSError:
            logger.critical("Socket error on port %s" % port)
            raise
        except Exception:
            logger.critical(traceback.format_exc())
            raise

        if self.daemon:
            try:
                self.daemon.start()
                try:
                    self.stop_flag.wait()
                except KeyboardInterrupt:
                    pass
                finally:
                    self.daemon.stop()
            except Exception:
                logger.critical(traceback.format_exc())
                raise

    def request_shutdown(self):
        if self.is_alive():
            self.stop_flag.set()

    def wait(self, timeout=None):
        self.proc.join(timeout)

    def is_alive(self):
        return self.proc.is_alive()


def check_subdomains(logger, config, routes, mp_context, log_handlers):
    paths = config.paths
    bind_address = config.bind_address

    host = config.server_host
    port = get_port()
    logger.debug("Going to use port %d to check subdomains" % port)

    wrapper = ServerProc(mp_context)
    wrapper.start(start_http_server, host, port, paths, routes,
                  bind_address, config, log_handlers)

    url = f"http://{host}:{port}/"
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
            logger.critical(f"Failed probing domain {domain}. {EDIT_HOSTS_HELP}")
            sys.exit(1)

    wrapper.request_shutdown()
    wrapper.wait()


def make_hosts_file(config, host):
    rv = ["# Start web-platform-tests hosts"]

    for domain in sorted(
        config.domains_set, key=lambda x: tuple(reversed(x.split(".")))
    ):
        rv.append("%s\t%s" % (host, domain))

    # Windows interpets the IP address 0.0.0.0 as non-existent, making it an
    # appropriate alias for non-existent hosts. However, UNIX-like systems
    # interpret the same address to mean any IP address, which is inappropraite
    # for this context. These systems do not reserve any value for this
    # purpose, so the inavailability of the domains must be taken for granted.
    #
    # https://github.com/web-platform-tests/wpt/issues/10560
    if platform.uname()[0] == "Windows":
        for not_domain in sorted(
            config.not_domains_set, key=lambda x: tuple(reversed(x.split(".")))
        ):
            rv.append("0.0.0.0\t%s" % not_domain)

    rv.append("# End web-platform-tests hosts")
    rv.append("")

    return "\n".join(rv)


def start_servers(logger, host, ports, paths, routes, bind_address, config,
                  mp_context, log_handlers, **kwargs):
    servers = defaultdict(list)
    for scheme, ports in ports.items():
        assert len(ports) == {"http": 2, "https": 2}.get(scheme, 1)

        # If trying to start HTTP/2.0 server, check compatibility
        if scheme == 'h2' and not http2_compatible():
            logger.error('Cannot start HTTP/2.0 server as the environment is not compatible. ' +
                         'Requires OpenSSL 1.0.2+')
            continue

        # Skip WebTransport over HTTP/3 server unless if is enabled explicitly.
        if scheme == 'webtransport-h3' and not kwargs.get("webtransport_h3"):
            continue

        for port in ports:
            if port is None:
                continue

            init_func = {
                "http": start_http_server,
                "http-private": start_http_server,
                "http-public": start_http_server,
                "https": start_https_server,
                "https-private": start_https_server,
                "https-public": start_https_server,
                "h2": start_http2_server,
                "ws": start_ws_server,
                "wss": start_wss_server,
                "webtransport-h3": start_webtransport_h3_server,
            }[scheme]

            server_proc = ServerProc(mp_context, scheme=scheme)
            server_proc.start(init_func, host, port, paths, routes, bind_address,
                              config, log_handlers, **kwargs)
            servers[scheme].append((port, server_proc))

    return servers


def startup_failed(logger):
    logger.critical(EDIT_HOSTS_HELP)
    sys.exit(1)


def start_http_server(logger, host, port, paths, routes, bind_address, config, **kwargs):
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
    except Exception as error:
        logger.critical(f"start_http_server: Caught exception from wptserve.WebTestHttpd: {error}")
        startup_failed(logger)


def start_https_server(logger, host, port, paths, routes, bind_address, config, **kwargs):
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
    except Exception as error:
        logger.critical(f"start_https_server: Caught exception from wptserve.WebTestHttpd: {error}")
        startup_failed(logger)


def start_http2_server(logger, host, port, paths, routes, bind_address, config, **kwargs):
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
    except Exception as error:
        logger.critical(f"start_http2_server: Caught exception from wptserve.WebTestHttpd: {error}")
        startup_failed(logger)


class WebSocketDaemon:
    def __init__(self, host, port, doc_root, handlers_root, bind_address, ssl_config):
        logger = logging.getLogger()
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
            logger.critical("Failed to start websocket server on port %s, "
                            "is something already using that port?" % port)
            raise OSError()
        assert all(item == ports[0] for item in ports)
        self.port = ports[0]
        self.started = False
        self.server_thread = None

    def start(self):
        self.started = True
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


def start_ws_server(logger, host, port, paths, routes, bind_address, config, **kwargs):
    try:
        return WebSocketDaemon(host,
                               str(port),
                               repo_root,
                               config.paths["ws_doc_root"],
                               bind_address,
                               ssl_config=None)
    except Exception as error:
        logger.critical(f"start_ws_server: Caught exception from WebSocketDomain: {error}")
        startup_failed(logger)


def start_wss_server(logger, host, port, paths, routes, bind_address, config, **kwargs):
    try:
        return WebSocketDaemon(host,
                               str(port),
                               repo_root,
                               config.paths["ws_doc_root"],
                               bind_address,
                               config.ssl_config)
    except Exception as error:
        logger.critical(f"start_wss_server: Caught exception from WebSocketDomain: {error}")
        startup_failed(logger)


def start_webtransport_h3_server(logger, host, port, paths, routes, bind_address, config, **kwargs):
    try:
        # TODO(bashi): Move the following import to the beginning of this file
        # once WebTransportH3Server is enabled by default.
        from webtransport.h3.webtransport_h3_server import WebTransportH3Server  # type: ignore
        return WebTransportH3Server(host=host,
                                    port=port,
                                    doc_root=paths["doc_root"],
                                    cert_path=config.ssl_config["cert_path"],
                                    key_path=config.ssl_config["key_path"],
                                    logger=logger)
    except Exception as error:
        logger.critical(
            f"Failed to start WebTransport over HTTP/3 server: {error}")
        sys.exit(0)


def start(logger, config, routes, mp_context, log_handlers, **kwargs):
    host = config["server_host"]
    ports = config.ports
    paths = config.paths
    bind_address = config["bind_address"]

    logger.debug("Using ports: %r" % ports)

    servers = start_servers(logger, host, ports, paths, routes, bind_address, config, mp_context,
                            log_handlers, **kwargs)

    return servers


def iter_servers(servers):
    for servers in servers.values():
        for port, server in servers:
            yield server


def _make_subdomains_product(s: Set[str], depth: int = 2) -> Set[str]:
    return {".".join(x) for x in chain(*(product(s, repeat=i) for i in range(1, depth+1)))}


_subdomains = {"www",
               "www1",
               "www2",
               "天気の良い日",
               "élève"}

_not_subdomains = {"nonexistent"}

_subdomains = _make_subdomains_product(_subdomains)

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
            "http-private": ["auto"],
            "http-public": ["auto"],
            "https": [8443, 8444],
            "https-private": ["auto"],
            "https-public": ["auto"],
            "ws": ["auto"],
            "wss": ["auto"],
            "webtransport-h3": ["auto"],
        },
        "check_subdomains": True,
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
        "aliases": [],
        "logging": {
            "level": "info",
            "suppress_handler_traceback": False
        }
    }

    computed_properties = ["ws_doc_root"] + config.ConfigBuilder.computed_properties

    def __init__(self, logger, *args, **kwargs):
        if "subdomains" not in kwargs:
            kwargs["subdomains"] = _subdomains
        if "not_subdomains" not in kwargs:
            kwargs["not_subdomains"] = _not_subdomains
        super().__init__(
            logger,
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

    def _get_paths(self, data):
        rv = super()._get_paths(data)
        rv["ws_doc_root"] = data["ws_doc_root"]
        return rv


def build_config(logger, override_path=None, config_cls=ConfigBuilder, **kwargs):
    rv = config_cls(logger)

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
        rv.logging["level"] = "DEBUG"

    setattr(rv, "inject_script", kwargs.get("inject_script"))

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
    parser.add_argument("--inject-script", default=None,
                        help="Path to script file to inject, useful for testing polyfills.")
    parser.add_argument("--alias_file", action="store", dest="alias_file",
                        help="File with entries for aliases/multiple doc roots. In form of `/ALIAS_NAME/, DOC_ROOT\\n`")
    parser.add_argument("--h2", action="store_true", dest="h2", default=None,
                        help=argparse.SUPPRESS)
    parser.add_argument("--no-h2", action="store_false", dest="h2", default=None,
                        help="Disable the HTTP/2.0 server")
    parser.add_argument("--webtransport-h3", action="store_true",
                        help="Enable WebTransport over HTTP/3 server")
    parser.add_argument("--exit-after-start", action="store_true", help="Exit after starting servers")
    parser.add_argument("--verbose", action="store_true", help="Enable verbose logging")
    parser.set_defaults(report=False)
    parser.set_defaults(is_wave=False)
    return parser


class MpContext:
    def __getattr__(self, name):
        return getattr(multiprocessing, name)


def get_logger(log_level, log_handlers):
    """Get a logger configured to log at level log_level

    If the logger has existing handlers the log_handlers argument is ignored.
    Otherwise the handlers in log_handlers are added to the logger. If there are
    no log_handlers passed and no configured handlers, a stream handler is added
    to the logger.

    Typically this is called once per process to set up logging in that process.

    :param log_level: - A string representing a log level e.g. "info"
    :param log_handlers: - Optional list of Handler objects.
    """
    logger = logging.getLogger()
    logger.setLevel(getattr(logging, log_level.upper()))
    if not logger.hasHandlers():
        if log_handlers is not None:
            for handler in log_handlers:
                logger.addHandler(handler)
        else:
            handler = logging.StreamHandler(sys.stdout)
            formatter = logging.Formatter("[%(asctime)s %(processName)s] %(levelname)s - %(message)s")
            handler.setFormatter(formatter)
            logger.addHandler(handler)
    return logger


def run(config_cls=ConfigBuilder, route_builder=None, mp_context=None, log_handlers=None,
        **kwargs):
    logger = get_logger("INFO", log_handlers)

    if mp_context is None:
        if hasattr(multiprocessing, "get_context"):
            mp_context = multiprocessing.get_context()
        else:
            mp_context = MpContext()

    with build_config(logger,
                      os.path.join(repo_root, "config.json"),
                      config_cls=config_cls,
                      **kwargs) as config:
        # This sets the right log level
        logger = get_logger(config.logging["level"], log_handlers)

        bind_address = config["bind_address"]

        if kwargs.get("alias_file"):
            with open(kwargs["alias_file"]) as alias_file:
                for line in alias_file:
                    alias, doc_root = (x.strip() for x in line.split(','))
                    config["aliases"].append({
                        'url-path': alias,
                        'local-dir': doc_root,
                    })

        if route_builder is None:
            route_builder = get_route_builder
        routes = route_builder(logger, config.aliases, config).get_routes()

        if config["check_subdomains"]:
            check_subdomains(logger, config, routes, mp_context, log_handlers)

        stash_address = None
        if bind_address:
            stash_address = (config.server_host, get_port(""))
            logger.debug("Going to use port %d for stash" % stash_address[1])

        with stash.StashServer(stash_address, authkey=str(uuid.uuid4())):
            servers = start(logger, config, routes, mp_context, log_handlers, **kwargs)

            if not kwargs.get("exit_after_start"):
                try:
                    # Periodically check if all the servers are alive
                    server_process_exited = False
                    while not server_process_exited:
                        for server in iter_servers(servers):
                            server.proc.join(1)
                            if not server.proc.is_alive():
                                server_process_exited = True
                                break
                except KeyboardInterrupt:
                    pass

            failed_subproc = 0
            for server in iter_servers(servers):
                logger.info('Status of subprocess "%s": running', server.proc.name)
                server.request_shutdown()

            for server in iter_servers(servers):
                server.wait(timeout=1)
                if server.proc.exitcode == 0:
                    logger.info('Status of subprocess "%s": exited correctly', server.proc.name)
                else:
                    subproc = server.proc
                    logger.warning('Status of subprocess "%s": failed. Exit with non-zero status: %d',
                                   subproc.name, subproc.exitcode)
                    failed_subproc += 1
            return failed_subproc


def main():
    kwargs = vars(get_parser().parse_args())
    return run(**kwargs)
