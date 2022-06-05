# mypy: allow-untyped-defs

import http.client as httplib
import sys
import logging
import traceback


global logger
logger = logging.getLogger("wave-api-handler")

class HttpHandler:
    def __init__(
        self,
        static_handler,
        sessions_api_handler,
        tests_api_handler,
        results_api_handler,
        devices_api_handler,
        general_api_handler,
        http_port,
        web_root
    ):
        self.static_handler = static_handler
        self.sessions_api_handler = sessions_api_handler
        self.tests_api_handler = tests_api_handler
        self.results_api_handler = results_api_handler
        self.general_api_handler = general_api_handler
        self.devices_api_handler = devices_api_handler
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
        path = path.split("?")[0]
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
        if api_name == "devices":
            self.devices_api_handler.handle_request(request, response)
            return

        self.general_api_handler.handle_request(request, response)

    def handle_static_file(self, request, response):
        self.static_handler.handle_request(request, response)

    def _remove_web_root(self, path):
        if self._web_root is not None:
            path = path[len(self._web_root):]
        return path


    def _proxy(self, request, response):
        host = 'localhost'
        port = int(self._http_port)
        uri = request.url_parts.path
        uri = uri + "?" + request.url_parts.query
        content_length = request.headers.get('Content-Length')
        data = ""
        if content_length is not None:
            data = request.raw_input.read(int(content_length))
        method = request.method

        headers = {}
        for key in request.headers:
            value = request.headers[key]
            headers[key.decode("utf-8")] = value.decode("utf-8")

        try:
            proxy_connection = httplib.HTTPConnection(host, port)
            proxy_connection.request(method, uri, data, headers)
            proxy_response = proxy_connection.getresponse()
            response.content = proxy_response.read()
            response.headers = proxy_response.getheaders()
            response.status = proxy_response.status

        except OSError:
            message = "Failed to perform proxy request"
            info = sys.exc_info()
            traceback.print_tb(info[2])
            logger.error(f"{message}: {info[0].__name__}: {info[1].args[0]}")
            response.status = 500
