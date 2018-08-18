import contextlib
import httplib
import json

from six import text_type


class HTTPRequest(object):
    def __init__(self, host, port):
        self.host = host
        self.port = port

    def head(self, path):
        return self._request("HEAD", path)

    def get(self, path):
        return self._request("GET", path)

    def post(self, path, body):
        return self._request("POST", path, body)

    @contextlib.contextmanager
    def _request(self, method, path, body=None):
        payload = None

        if body is not None:
            try:
                payload = json.dumps(body)
            except ValueError:
                raise ValueError("Failed to encode request body as JSON: {}".format(
                    json.dumps(body, indent=2)))

            if isinstance(payload, text_type):
                payload = body.encode("utf-8")

        conn = httplib.HTTPConnection(self.host, self.port)
        try:
            conn.request(method, path, payload)
            yield conn.getresponse()
        finally:
            conn.close()
