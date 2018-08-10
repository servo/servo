from six.moves import BaseHTTPServer
import errno
import os
import socket
from six.moves.socketserver import ThreadingMixIn
import ssl
import sys
import threading
import time
import traceback
from six import binary_type, text_type
import uuid
from collections import OrderedDict

from h2.config import H2Configuration
from h2.connection import H2Connection
from h2.events import RequestReceived, ConnectionTerminated

from six.moves.urllib.parse import urlsplit, urlunsplit

from . import routes as default_routes
from .config import ConfigBuilder
from .logger import get_logger
from .request import Server, Request
from .response import Response, H2Response
from .router import Router
from .utils import HTTPException
from .constants import h2_headers

"""HTTP server designed for testing purposes.

The server is designed to provide flexibility in the way that
requests are handled, and to provide control both of exactly
what bytes are put on the wire for the response, and in the
timing of sending those bytes.

The server is based on the stdlib HTTPServer, but with some
notable differences in the way that requests are processed.
Overall processing is handled by a WebTestRequestHandler,
which is a subclass of BaseHTTPRequestHandler. This is responsible
for parsing the incoming request. A RequestRewriter is then
applied and may change the request data if it matches a
supplied rule.

Once the request data had been finalised, Request and Reponse
objects are constructed. These are used by the other parts of the
system to read information about the request and manipulate the
response.

Each request is handled by a particular handler function. The
mapping between Request and the appropriate handler is determined
by a Router. By default handlers are installed to interpret files
under the document root with .py extensions as executable python
files (see handlers.py for the api for such files), .asis files as
bytestreams to be sent literally and all other files to be served
statically.

The handler functions are responsible for either populating the
fields of the response object, which will then be written when the
handler returns, or for directly writing to the output stream.
"""


class RequestRewriter(object):
    def __init__(self, rules):
        """Object for rewriting the request path.

        :param rules: Initial rules to add; a list of three item tuples
                      (method, input_path, output_path), defined as for
                      register()
        """
        self.rules = {}
        for rule in reversed(rules):
            self.register(*rule)
        self.logger = get_logger()

    def register(self, methods, input_path, output_path):
        """Register a rewrite rule.

        :param methods: Set of methods this should match. "*" is a
                        special value indicating that all methods should
                        be matched.

        :param input_path: Path to match for the initial request.

        :param output_path: Path to replace the input path with in
                            the request.
        """
        if isinstance(methods, (binary_type, text_type)):
            methods = [methods]
        self.rules[input_path] = (methods, output_path)

    def rewrite(self, request_handler):
        """Rewrite the path in a BaseHTTPRequestHandler instance, if
           it matches a rule.

        :param request_handler: BaseHTTPRequestHandler for which to
                                rewrite the request.
        """
        split_url = urlsplit(request_handler.path)
        if split_url.path in self.rules:
            methods, destination = self.rules[split_url.path]
            if "*" in methods or request_handler.command in methods:
                self.logger.debug("Rewriting request path %s to %s" %
                             (request_handler.path, destination))
                new_url = list(split_url)
                new_url[2] = destination
                new_url = urlunsplit(new_url)
                request_handler.path = new_url


class WebTestServer(ThreadingMixIn, BaseHTTPServer.HTTPServer):
    allow_reuse_address = True
    acceptable_errors = (errno.EPIPE, errno.ECONNABORTED)
    request_queue_size = 2000

    # Ensure that we don't hang on shutdown waiting for requests
    daemon_threads = True

    def __init__(self, server_address, request_handler_cls,
                 router, rewriter, bind_address,
                 config=None, use_ssl=False, key_file=None, certificate=None,
                 encrypt_after_connect=False, latency=None, http2=False, **kwargs):
        """Server for HTTP(s) Requests

        :param server_address: tuple of (server_name, port)

        :param request_handler_cls: BaseHTTPRequestHandler-like class to use for
                                    handling requests.

        :param router: Router instance to use for matching requests to handler
                       functions

        :param rewriter: RequestRewriter-like instance to use for preprocessing
                         requests before they are routed

        :param config: Dictionary holding environment configuration settings for
                       handlers to read, or None to use the default values.

        :param use_ssl: Boolean indicating whether the server should use SSL

        :param key_file: Path to key file to use if SSL is enabled.

        :param certificate: Path to certificate to use if SSL is enabled.

        :param encrypt_after_connect: For each connection, don't start encryption
                                      until a CONNECT message has been received.
                                      This enables the server to act as a
                                      self-proxy.

        :param bind_address True to bind the server to both the IP address and
                            port specified in the server_address parameter.
                            False to bind the server only to the port in the
                            server_address parameter, but not to the address.
        :param latency: Delay in ms to wait before seving each response, or
                        callable that returns a delay in ms
        """
        self.router = router
        self.rewriter = rewriter

        self.scheme = "http2" if http2 else "https" if use_ssl else "http"
        self.logger = get_logger()

        self.latency = latency

        if bind_address:
            hostname_port = server_address
        else:
            hostname_port = ("",server_address[1])

        #super doesn't work here because BaseHTTPServer.HTTPServer is old-style
        BaseHTTPServer.HTTPServer.__init__(self, hostname_port, request_handler_cls, **kwargs)

        if config is not None:
            Server.config = config
        else:
            self.logger.debug("Using default configuration")
            with ConfigBuilder(browser_host=server_address[0],
                               ports={"http": [self.server_address[1]]}) as config:
                assert config["ssl_config"] is None
                Server.config = config



        self.key_file = key_file
        self.certificate = certificate
        self.encrypt_after_connect = use_ssl and encrypt_after_connect

        if use_ssl and not encrypt_after_connect:
            if http2:
                ssl_context = ssl.create_default_context(purpose=ssl.Purpose.CLIENT_AUTH)
                ssl_context.load_cert_chain(keyfile=self.key_file, certfile=self.certificate)
                ssl_context.set_alpn_protocols(['h2'])
                self.socket = ssl_context.wrap_socket(self.socket,
                                                      server_side=True)

            else:
                self.socket = ssl.wrap_socket(self.socket,
                                              keyfile=self.key_file,
                                              certfile=self.certificate,
                                              server_side=True)

    def handle_error(self, request, client_address):
        error = sys.exc_info()[1]

        if ((isinstance(error, socket.error) and
             isinstance(error.args, tuple) and
             error.args[0] in self.acceptable_errors) or
            (isinstance(error, IOError) and
             error.errno in self.acceptable_errors)):
            pass  # remote hang up before the result is sent
        else:
            self.logger.error(traceback.format_exc())


class BaseWebTestRequestHandler(BaseHTTPServer.BaseHTTPRequestHandler):
    """RequestHandler for WebTestHttpd"""

    def __init__(self, *args, **kwargs):
        self.logger = get_logger()
        BaseHTTPServer.BaseHTTPRequestHandler.__init__(self, *args, **kwargs)

    def finish_handling(self, request_line_is_valid, response_cls):
            self.server.rewriter.rewrite(self)

            request = Request(self)
            response = response_cls(self, request)

            if request.method == "CONNECT":
                self.handle_connect(response)
                return

            if not request_line_is_valid:
                response.set_error(414)
                response.write()
                return

            self.logger.debug("%s %s" % (request.method, request.request_path))
            handler = self.server.router.get_handler(request)

            # If the handler we used for the request had a non-default base path
            # set update the doc_root of the request to reflect this
            if hasattr(handler, "base_path") and handler.base_path:
                request.doc_root = handler.base_path
            if hasattr(handler, "url_base") and handler.url_base != "/":
                request.url_base = handler.url_base

            if self.server.latency is not None:
                if callable(self.server.latency):
                    latency = self.server.latency()
                else:
                    latency = self.server.latency
                self.logger.warning("Latency enabled. Sleeping %i ms" % latency)
                time.sleep(latency / 1000.)

            if handler is None:
                response.set_error(404)
            else:
                try:
                    handler(request, response)
                except HTTPException as e:
                    response.set_error(e.code, e.message)
                except Exception as e:
                    message = str(e)
                    if message:
                        err = [message]
                    else:
                        err = []
                    err.append(traceback.format_exc())
                    response.set_error(500, "\n".join(err))
            self.logger.debug("%i %s %s (%s) %i" % (response.status[0],
                                                    request.method,
                                                    request.request_path,
                                                    request.headers.get('Referer'),
                                                    request.raw_input.length))

            if not response.writer.content_written:
                response.write()

            # If a python handler has been used, the old ones won't send a END_STR data frame, so this
            # allows for backwards compatibility by accounting for these handlers that don't close streams
            if isinstance(response, H2Response) and not response.writer.content_written:
                response.writer.write_content('', last=True)

            # If we want to remove this in the future, a solution is needed for
            # scripts that produce a non-string iterable of content, since these
            # can't set a Content-Length header. A notable example of this kind of
            # problem is with the trickle pipe i.e. foo.js?pipe=trickle(d1)
            if response.close_connection:
                self.close_connection = True

            if not self.close_connection:
                # Ensure that the whole request has been read from the socket
                request.raw_input.read()

    def handle_connect(self, response):
        self.logger.debug("Got CONNECT")
        response.status = 200
        response.write()
        if self.server.encrypt_after_connect:
            self.logger.debug("Enabling SSL for connection")
            self.request = ssl.wrap_socket(self.connection,
                                           keyfile=self.server.key_file,
                                           certfile=self.server.certificate,
                                           server_side=True)
            self.setup()
        return


class Http2WebTestRequestHandler(BaseWebTestRequestHandler):
    protocol_version = "HTTP/2.0"

    def handle_one_request(self):
        """
        This is the main HTTP/2.0 Handler. When a browser opens a connection to the server
        on the HTTP/2.0 port, Because there can be multiple H2 connections active at the same
        time, a UUID is created for each so that it is easier to tell them apart in the logs.
        """

        config = H2Configuration(client_side=False)
        self.conn = H2ConnectionGuard(H2Connection(config=config))
        self.close_connection = False

        # Generate a UUID to make it easier to distinguish different H2 connection debug messages
        uid = uuid.uuid4()

        self.logger.debug('(%s) Initiating h2 Connection' % uid)

        with self.conn as connection:
            connection.initiate_connection()
            data = connection.data_to_send()

        self.request.sendall(data)

        self.request_threads = []

        # TODO Need to do some major work on multithreading. Current idea is to have a thread per stream
        # so that processing of the request can start from the first frame.

        while not self.close_connection:
            try:
                # This size may need to be made variable based on remote settings?
                data = self.request.recv(65535)

                with self.conn as connection:
                    events = connection.receive_data(data)

                self.logger.debug('(%s) Events: ' % (uid) + str(events))

                for event in events:
                    if isinstance(event, RequestReceived):
                        self.logger.debug('(%s) Parsing RequestReceived' % (uid))
                        self._h2_parse_request(event)
                        t = threading.Thread(target=BaseWebTestRequestHandler.finish_handling, args=(self, True, H2Response))
                        self.request_threads.append(t)
                        t.start()
                    if isinstance(event, ConnectionTerminated):
                        self.logger.debug('(%s) Connection terminated by remote peer ' % (uid))
                        self.close_connection = True

            except (socket.timeout, socket.error) as e:
                self.logger.debug('(%s) ERROR - Closing Connection - \n%s' % (uid, str(e)))
                self.close_connection = True
                for t in self.request_threads:
                    t.join()

    def _h2_parse_request(self, event):
        self.headers = H2Headers(event.headers)
        self.command = self.headers['method']
        self.path = self.headers['path']
        self.h2_stream_id = event.stream_id

        # TODO Need to figure out what to do with this thing as it is no longer used
        # For now I can just leave it be as it does not affect anything
        self.raw_requestline = ''


class H2ConnectionGuard(object):
    lock = threading.Lock()

    def __init__(self, obj):
        self.obj = obj

    def __enter__(self):
        self.lock.acquire()
        return self.obj

    def __exit__(self, exception_type, exception_value, traceback):
        self.lock.release()


class H2Headers(dict):
    def __init__(self, headers):
        self.raw_headers = OrderedDict()
        for key, val in headers:
            self.raw_headers[key] = val
            dict.__setitem__(self, self._convert_h2_header_to_h1(key), val)

    def _convert_h2_header_to_h1(self, header_key):
        if header_key[1:] in h2_headers and header_key[0] == ':':
            return header_key[1:]
        else:
            return header_key

    # TODO This does not seem relevant for H2 headers, so using a dummy function for now
    def getallmatchingheaders(self, header):
        return ['dummy function']


class Http1WebTestRequestHandler(BaseWebTestRequestHandler):
    protocol_version = "HTTP/1.1"

    def handle_one_request(self):
        response = None

        try:
            self.close_connection = False

            request_line_is_valid = self.get_request_line()

            if self.close_connection:
                return

            request_is_valid = self.parse_request()
            if not request_is_valid:
                #parse_request() actually sends its own error responses
                return

            self.finish_handling(request_line_is_valid, Response)

        except socket.timeout as e:
            self.log_error("Request timed out: %r", e)
            self.close_connection = True
            return

        except Exception as e:
            err = traceback.format_exc()
            if response:
                response.set_error(500, err)
                response.write()
            self.logger.error(err)

    def get_request_line(self):
        try:
            self.raw_requestline = self.rfile.readline(65537)
        except socket.error:
            self.close_connection = True
            return False
        if len(self.raw_requestline) > 65536:
            self.requestline = ''
            self.request_version = ''
            self.command = ''
            return False
        if not self.raw_requestline:
            self.close_connection = True
        return True

class WebTestHttpd(object):
    """
    :param host: Host from which to serve (default: 127.0.0.1)
    :param port: Port from which to serve (default: 8000)
    :param server_cls: Class to use for the server (default depends on ssl vs non-ssl)
    :param handler_cls: Class to use for the RequestHandler
    :param use_ssl: Use a SSL server if no explicit server_cls is supplied
    :param key_file: Path to key file to use if ssl is enabled
    :param certificate: Path to certificate file to use if ssl is enabled
    :param encrypt_after_connect: For each connection, don't start encryption
                                  until a CONNECT message has been received.
                                  This enables the server to act as a
                                  self-proxy.
    :param router_cls: Router class to use when matching URLs to handlers
    :param doc_root: Document root for serving files
    :param routes: List of routes with which to initialize the router
    :param rewriter_cls: Class to use for request rewriter
    :param rewrites: List of rewrites with which to initialize the rewriter_cls
    :param config: Dictionary holding environment configuration settings for
                   handlers to read, or None to use the default values.
    :param bind_address: Boolean indicating whether to bind server to IP address.
    :param latency: Delay in ms to wait before seving each response, or
                    callable that returns a delay in ms

    HTTP server designed for testing scenarios.

    Takes a router class which provides one method get_handler which takes a Request
    and returns a handler function.

    .. attribute:: host

      The host name or ip address of the server

    .. attribute:: port

      The port on which the server is running

    .. attribute:: router

      The Router object used to associate requests with resources for this server

    .. attribute:: rewriter

      The Rewriter object used for URL rewriting

    .. attribute:: use_ssl

      Boolean indicating whether the server is using ssl

    .. attribute:: started

      Boolean indictaing whether the server is running

    """
    def __init__(self, host="127.0.0.1", port=8000,
                 server_cls=None, handler_cls=Http1WebTestRequestHandler,
                 use_ssl=False, key_file=None, certificate=None, encrypt_after_connect=False,
                 router_cls=Router, doc_root=os.curdir, routes=None,
                 rewriter_cls=RequestRewriter, bind_address=True, rewrites=None,
                 latency=None, config=None, http2=False):

        if routes is None:
            routes = default_routes.routes

        self.host = host

        self.router = router_cls(doc_root, routes)
        self.rewriter = rewriter_cls(rewrites if rewrites is not None else [])

        self.use_ssl = use_ssl
        self.http2 = http2
        self.logger = get_logger()

        if server_cls is None:
            server_cls = WebTestServer

        if use_ssl:
            if not os.path.exists(key_file):
                raise ValueError("SSL certificate not found: {}".format(key_file))
            if not os.path.exists(certificate):
                raise ValueError("SSL key not found: {}".format(certificate))

        try:
            self.httpd = server_cls((host, port),
                                    handler_cls,
                                    self.router,
                                    self.rewriter,
                                    config=config,
                                    bind_address=bind_address,
                                    use_ssl=use_ssl,
                                    key_file=key_file,
                                    certificate=certificate,
                                    encrypt_after_connect=encrypt_after_connect,
                                    latency=latency,
                                    http2=http2)
            self.started = False

            _host, self.port = self.httpd.socket.getsockname()
        except Exception:
            self.logger.error("Failed to start HTTP server. "
                              "You may need to edit /etc/hosts or similar, see README.md.")
            raise

    def start(self, block=False):
        """Start the server.

        :param block: True to run the server on the current thread, blocking,
                      False to run on a separate thread."""
        http_type = "http2" if self.http2 else "https" if self.use_ssl else "http"
        self.logger.info("Starting %s server on %s:%s" % (http_type, self.host, self.port))
        self.started = True
        if block:
            self.httpd.serve_forever()
        else:
            self.server_thread = threading.Thread(target=self.httpd.serve_forever)
            self.server_thread.setDaemon(True)  # don't hang on exit
            self.server_thread.start()

    def stop(self):
        """
        Stops the server.

        If the server is not running, this method has no effect.
        """
        if self.started:
            try:
                self.httpd.shutdown()
                self.httpd.server_close()
                self.server_thread.join()
                self.server_thread = None
                self.logger.info("Stopped http server on %s:%s" % (self.host, self.port))
            except AttributeError:
                pass
            self.started = False
        self.httpd = None

    def get_url(self, path="/", query=None, fragment=None):
        if not self.started:
            return None

        return urlunsplit(("http" if not self.use_ssl else "https",
                           "%s:%s" % (self.host, self.port),
                           path, query, fragment))
