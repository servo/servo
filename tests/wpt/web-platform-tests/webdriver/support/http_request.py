import contextlib
import httplib

class HTTPRequest(object):
    def __init__(self, host, port):
        self.host = host
        self.port = port

    def head(self, path):
        return self._request("HEAD", path)

    def get(self, path):
        return self._request("GET", path)

    @contextlib.contextmanager
    def _request(self, method, path):
        conn = httplib.HTTPConnection(self.host, self.port)
        try:
            conn.request(method, path)
            yield conn.getresponse()
        finally:
            conn.close()
