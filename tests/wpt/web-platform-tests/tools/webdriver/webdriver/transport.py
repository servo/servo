# mypy: allow-untyped-defs

import json
import select

from http.client import HTTPConnection
from typing import Dict, List, Mapping, Sequence, Tuple
from urllib import parse as urlparse

from . import error

"""Implements HTTP transport for the WebDriver wire protocol."""


missing = object()


class ResponseHeaders(Mapping[str, str]):
    """Read-only dictionary-like API for accessing response headers.

    This class:
      * Normalizes the header keys it is built with to lowercase (such that
        iterating the items will return lowercase header keys).
      * Has case-insensitive header lookup.
      * Always returns all header values that have the same name, separated by
        commas.
    """
    def __init__(self, items: Sequence[Tuple[str, str]]):
        self.headers_dict: Dict[str, List[str]] = {}
        for key, value in items:
            key = key.lower()
            if key not in self.headers_dict:
                self.headers_dict[key] = []
            self.headers_dict[key].append(value)

    def __getitem__(self, key):
        """Get all headers of a certain (case-insensitive) name. If there is
        more than one, the values are returned comma separated"""
        values = self.headers_dict[key.lower()]
        if len(values) == 1:
            return values[0]
        else:
            return ", ".join(values)

    def get_list(self, key, default=missing):
        """Get all the header values for a particular field name as a list"""
        try:
            return self.headers_dict[key.lower()]
        except KeyError:
            if default is not missing:
                return default
            else:
                raise

    def __iter__(self):
        yield from self.headers_dict

    def __len__(self):
        return len(self.headers_dict)


class Response:
    """
    Describes an HTTP response received from a remote end whose
    body has been read and parsed as appropriate.
    """

    def __init__(self, status, body, headers):
        self.status = status
        self.body = body
        self.headers = headers

    def __repr__(self):
        cls_name = self.__class__.__name__
        if self.error:
            return f"<{cls_name} status={self.status} error={repr(self.error)}>"
        return f"<{cls_name: }tatus={self.status} body={json.dumps(self.body)}>"

    def __str__(self):
        return json.dumps(self.body, indent=2)

    @property
    def error(self):
        if self.status != 200:
            return error.from_response(self)
        return None

    @classmethod
    def from_http(cls, http_response, decoder=json.JSONDecoder, **kwargs):
        try:
            body = json.load(http_response, cls=decoder, **kwargs)
            headers = ResponseHeaders(http_response.getheaders())
        except ValueError:
            raise ValueError("Failed to decode response body as JSON:\n" +
                http_response.read())

        return cls(http_response.status, body, headers)


class HTTPWireProtocol:
    """
    Transports messages (commands and responses) over the WebDriver
    wire protocol.

    Complex objects, such as ``webdriver.Element``, ``webdriver.Frame``,
    and ``webdriver.Window`` are by default not marshaled to enable
    use of `session.transport.send` in WPT tests::

        session = webdriver.Session("127.0.0.1", 4444)
        response = transport.send("GET", "element/active", None)
        print response.body["value"]
        # => {u'element-6066-11e4-a52e-4f735466cecf': u'<uuid>'}

    Automatic marshaling is provided by ``webdriver.protocol.Encoder``
    and ``webdriver.protocol.Decoder``, which can be passed in to
    ``HTTPWireProtocol.send`` along with a reference to the current
    ``webdriver.Session``::

        session = webdriver.Session("127.0.0.1", 4444)
        response = transport.send("GET", "element/active", None,
            encoder=protocol.Encoder, decoder=protocol.Decoder,
            session=session)
        print response.body["value"]
        # => webdriver.Element
    """

    def __init__(self, host, port, url_prefix="/"):
        """
        Construct interface for communicating with the remote server.

        :param url: URL of remote WebDriver server.
        :param wait: Duration to wait for remote to appear.
        """
        self.host = host
        self.port = port
        self.url_prefix = url_prefix
        self._conn = None
        self._last_request_is_blocked = False

    def __del__(self):
        self.close()

    def close(self):
        """Closes the current HTTP connection, if there is one."""
        if self._conn:
            try:
                self._conn.close()
            except OSError:
                # The remote closed the connection
                pass
        self._conn = None

    @property
    def connection(self):
        """Gets the current HTTP connection, or lazily creates one."""
        if not self._conn:
            conn_kwargs = {}
            # We are not setting an HTTP timeout other than the default when the
            # connection its created. The send method has a timeout value if needed.
            self._conn = HTTPConnection(self.host, self.port, **conn_kwargs)

        return self._conn

    def url(self, suffix):
        """
        From the relative path to a command end-point,
        craft a full URL suitable to be used in a request to the HTTPD.
        """
        return urlparse.urljoin(self.url_prefix, suffix)

    def send(self,
             method,
             uri,
             body=None,
             headers=None,
             encoder=json.JSONEncoder,
             decoder=json.JSONDecoder,
             timeout=None,
             **codec_kwargs):
        """
        Send a command to the remote.

        The request `body` must be JSON serialisable unless a
        custom `encoder` has been provided.  This means complex
        objects such as ``webdriver.Element``, ``webdriver.Frame``,
        and `webdriver.Window`` are not automatically made
        into JSON.  This behaviour is, however, provided by
        ``webdriver.protocol.Encoder``, should you want it.

        Similarly, the response body is returned au natural
        as plain JSON unless a `decoder` that converts web
        element references to ``webdriver.Element`` is provided.
        Use ``webdriver.protocol.Decoder`` to achieve this behaviour.

        The client will attempt to use persistent HTTP connections.

        :param method: `GET`, `POST`, or `DELETE`.
        :param uri: Relative endpoint of the requests URL path.
        :param body: Body of the request.  Defaults to an empty
            dictionary if ``method`` is `POST`.
        :param headers: Additional dictionary of headers to include
            in the request.
        :param encoder: JSON encoder class, which defaults to
            ``json.JSONEncoder`` unless specified.
        :param decoder: JSON decoder class, which defaults to
            ``json.JSONDecoder`` unless specified.
        :param codec_kwargs: Surplus arguments passed on to `encoder`
            and `decoder` on construction.

        :return: Instance of ``webdriver.transport.Response``
            describing the HTTP response received from the remote end.

        :raises ValueError: If `body` or the response body are not
            JSON serialisable.
        """
        if body is None and method == "POST":
            body = {}

        payload = None
        if body is not None:
            try:
                payload = json.dumps(body, cls=encoder, **codec_kwargs)
            except ValueError:
                raise ValueError("Failed to encode request body as JSON:\n"
                    "%s" % json.dumps(body, indent=2))

        # When the timeout triggers, the TestRunnerManager thread will reuse
        # this connection to check if the WebDriver its alive and we may end
        # raising an httplib.CannotSendRequest exception if the WebDriver is
        # not responding and this httplib.request() call is blocked on the
        # runner thread. We use the boolean below to check for that and restart
        # the connection in that case.
        self._last_request_is_blocked = True
        response = self._request(method, uri, payload, headers, timeout=None)
        self._last_request_is_blocked = False
        return Response.from_http(response, decoder=decoder, **codec_kwargs)

    def _request(self, method, uri, payload, headers=None, timeout=None):
        if isinstance(payload, str):
            payload = payload.encode("utf-8")

        if headers is None:
            headers = {}
        headers.update({"Connection": "keep-alive"})

        url = self.url(uri)

        if self._last_request_is_blocked or self._has_unread_data():
            self.close()

        self.connection.request(method, url, payload, headers)

        # timeout for request has to be set just before calling httplib.getresponse()
        # and the previous value restored just after that, even on exception raised
        try:
            if timeout:
                previous_timeout = self._conn.gettimeout()
                self._conn.settimeout(timeout)
            response = self.connection.getresponse()
        finally:
            if timeout:
                self._conn.settimeout(previous_timeout)

        return response

    def _has_unread_data(self):
        return self._conn and self._conn.sock and select.select([self._conn.sock], [], [], 0)[0]
