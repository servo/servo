import contextlib
import json

from http.client import HTTPConnection


class HTTPRequest(object):
    def __init__(self, host: str, port: int):
        self.host = host
        self.port = port

    def head(self, path: str):
        return self._request("HEAD", path)

    def get(self, path: str):
        return self._request("GET", path)

    def post(self, path: str, body):
        return self._request("POST", path, body)

    @contextlib.contextmanager
    def _request(self, method: str, path: str, body=None):
        payload = None

        if body is not None:
            try:
                payload = json.dumps(body)
            except ValueError:
                raise ValueError("Failed to encode request body as JSON: {}".format(
                    json.dumps(body, indent=2)))

            if isinstance(payload, str):
                payload = body.encode("utf-8")

        conn = HTTPConnection(self.host, self.port)
        try:
            conn.request(method, path, payload)
            yield conn.getresponse()
        finally:
            conn.close()
