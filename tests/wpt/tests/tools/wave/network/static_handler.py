# mypy: allow-untyped-defs

import os


class StaticHandler:
    def __init__(self, web_root, http_port, https_port):
        self.static_dir = os.path.join(
            os.getcwd(), "tools/wave/www")
        self._web_root = web_root
        self._http_port = http_port
        self._https_port = https_port

    def handle_request(self, request, response):
        file_path = request.request_path

        if self._web_root is not None:
            if not file_path.startswith(self._web_root):
                response.status = 404
                return
            file_path = file_path[len(self._web_root):]

        if file_path == "":
            file_path = "index.html"

        file_path = file_path.split("?")[0]
        file_path = os.path.join(self.static_dir, file_path)

        if not os.path.exists(file_path):
            response.status = 404
            return

        headers = []

        content_types = {
            "html": "text/html",
            "js": "text/javascript",
            "css": "text/css",
            "jpg": "image/jpeg",
            "jpeg": "image/jpeg",
            "ttf": "font/ttf",
            "woff": "font/woff",
            "woff2": "font/woff2"
        }

        headers.append(
            ("Content-Type", content_types[file_path.split(".")[-1]]))

        data = None
        with open(file_path, "rb") as file:
            data = file.read()

        if file_path.split("/")[-1] == "wave-service.js":
            data = data.decode("UTF-8")
            data = data.replace("{{WEB_ROOT}}", str(self._web_root))
            data = data.replace("{{HTTP_PORT}}", str(self._http_port))
            data = data.replace("{{HTTPS_PORT}}", str(self._https_port))

        response.content = data
        response.headers = headers
