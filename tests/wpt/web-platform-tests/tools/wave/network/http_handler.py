from __future__ import unicode_literals
try:
    import http.client as httplib
except ImportError:
    import httplib
import sys
import traceback


class HttpHandler(object):
    def __init__(
        self,
        static_handler=None,
        sessions_api_handler=None,
        tests_api_handler=None,
        results_api_handler=None,
        http_port=None,
        web_root=None
    ):
        self.static_handler = static_handler
        self.sessions_api_handler = sessions_api_handler
        self.tests_api_handler = tests_api_handler
        self.results_api_handler = results_api_handler
        self._http_port = http_port
        self._web_root = web_root

    def handle_request(self, request, response):
        response.headers = [
            ("Access-Control-Allow-Origin", "*"),
            ("Access-Control-Allow-Headers", "*"),
            ("Access-Control-Allow-Methods", "*")
        ]
        if request.method == "OPTIONS":
            return

        path = self._remove_web_root(request.request_path)

        is_api_call = False
        for index, part in enumerate(path.split("/")):
            if index > 2:
                break
            if part != "api":
                continue

            is_api_call = True

        if (is_api_call):
            if request.url_parts.scheme == "https":
                self._proxy(request, response)
                return
            self.handle_api(request, response)
        else:
            self.handle_static_file(request, response)

    def handle_api(self, request, response):
        path = self._remove_web_root(request.request_path)
        api_name = path.split("/")[1]

        if api_name is None:
            return

        if api_name == "sessions":
            self.sessions_api_handler.handle_request(request, response)
            return
        if api_name == "tests":
            self.tests_api_handler.handle_request(request, response)
            return
        if api_name == "results":
            self.results_api_handler.handle_request(request, response)
            return

    def handle_static_file(self, request, response):
        self.static_handler.handle_request(request, response)

    def _remove_web_root(self, path):
        if self._web_root is not None:
            path = path[len(self._web_root):]
        return path


    def _proxy(self, request, response):
        host = 'localhost'
        port = str(self._http_port)
        uri = request.url_parts.path
        uri = uri + "?" + request.url_parts.query
        data = request.raw_input.read(request.headers.get('Content-Length'))
        method = request.method

        try:
            proxy_connection = httplib.HTTPConnection(host, port)
            proxy_connection.request(method, uri, data, request.headers)
            proxy_response = proxy_connection.getresponse()
            response.content = proxy_response.read()
            response.headers = proxy_response.getheaders()
            response.status = proxy_response.status

        except IOError:
            info = sys.exc_info()
            traceback.print_tb(info[2])
            print("Failed to perform proxy request: " +
                info[0].__name__ + ": " + str(info[1].args[0]))
            response.status = 500
