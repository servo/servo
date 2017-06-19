import httplib
import json
import urlparse

class Response(object):
    """Describes an HTTP response received from a remote en"Describes an HTTP
    response received from a remote end whose body has been read and parsed as
    appropriate."""
    def __init__(self, status, body):
        self.status = status
        self.body = body

    def __repr__(self):
        return "wdclient.Response(status=%d, body=%s)" % (self.status, self.body)

    @classmethod
    def from_http_response(cls, http_response):
        status = http_response.status
        body = http_response.read()

        # SpecID: dfn-send-a-response
        #
        # > 3. Set the response's header with name and value with the following
        # >    values:
        # >
        # >    "Content-Type"
        # >       "application/json; charset=utf-8"
        # >    "cache-control"
        # >       "no-cache"
        assert http_response.getheader("Content-Type") == "application/json; charset=utf-8"
        assert http_response.getheader("Cache-Control") == "no-cache"

        if body:
            body = json.loads(body)

            # SpecID: dfn-send-a-response
            #
            # > 4. If data is not null, let response's body be a JSON Object
            #      with a key `value` set to the JSON Serialization of data.
            assert "value" in body

        return cls(status, body)


class HTTPWireProtocol(object):
    """Transports messages (commands and responses) over the WebDriver
    wire protocol.
    """

    def __init__(self, host, port, url_prefix="/", timeout=None):
        """Construct interface for communicating with the remote server.

        :param url: URL of remote WebDriver server.
        :param wait: Duration to wait for remote to appear.
        """

        self.host = host
        self.port = port
        self.url_prefix = url_prefix

        self._timeout = timeout

    def url(self, suffix):
        return urlparse.urljoin(self.path_prefix, suffix)

    def send(self, method, url, body=None, headers=None):
        """Send a command to the remote.

        :param method: "POST" or "GET".
        :param url: "command part" of the requests URL path
        :param body: Body of the request.  Defaults to an empty dictionary
            if ``method`` is "POST".
        :param headers: Additional headers to include in the request.
        :return: an instance of wdclient.Response describing the HTTP response
            received from the remote end.
        """

        if body is None and method == "POST":
            body = {}

        if isinstance(body, dict):
            body = json.dumps(body)

        if isinstance(body, unicode):
            body = body.encode("utf-8")

        if headers is None:
            headers = {}

        url = self.url_prefix + url

        kwargs = {}
        if self._timeout is not None:
            kwargs["timeout"] = self._timeout

        conn = httplib.HTTPConnection(
            self.host, self.port, strict=True, **kwargs)
        conn.request(method, url, body, headers)

        try:
            response = Response.from_http_response(conn.getresponse())
        finally:
            conn.close()

        return response
