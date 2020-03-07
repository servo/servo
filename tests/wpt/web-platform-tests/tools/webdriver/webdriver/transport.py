import json
import select

from six import text_type, PY3
from six.moves.http_client import HTTPConnection
from six.moves.urllib import parse as urlparse

from . import error

"""Implements HTTP transport for the WebDriver wire protocol."""


class Response(object):
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
            return "<%s status=%s error=%s>" % (cls_name, self.status, repr(self.error))
        return "<% status=%s body=%s>" % (cls_name, self.status, json.dumps(self.body))

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
            headers = dict(http_response.getheaders())
        except ValueError:
            raise ValueError("Failed to decode response body as JSON:\n" +
                http_response.read())

        return cls(http_response.status, body, headers)


class HTTPWireProtocol(object):
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
            self._conn.close()

    @property
    def connection(self):
        """Gets the current HTTP connection, or lazily creates one."""
        if not self._conn:
            conn_kwargs = {}
            if not PY3:
                conn_kwargs["strict"] = True
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
        if isinstance(payload, text_type):
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
