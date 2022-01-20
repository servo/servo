from __future__ import absolute_import
from __future__ import unicode_literals

from .api_handler import ApiHandler

TOKEN_LENGTH = 36


class GeneralApiHandler(ApiHandler):
    def __init__(
        self,
        web_root,
        read_sessions_enabled,
        import_results_enabled,
        reports_enabled,
        version_string,
        test_type_selection_enabled,
        test_file_selection_enabled
    ):
        super(GeneralApiHandler, self).__init__(web_root)
        self.read_sessions_enabled = read_sessions_enabled
        self.import_results_enabled = import_results_enabled
        self.reports_enabled = reports_enabled
        self.version_string = version_string
        self.test_type_selection_enabled = test_type_selection_enabled
        self.test_file_selection_enabled = test_file_selection_enabled

    def read_status(self):
        try:
            return {
                "format": "application/json",
                "data": {
                    "version_string": self.version_string,
                    "read_sessions_enabled": self.read_sessions_enabled,
                    "import_results_enabled": self.import_results_enabled,
                    "reports_enabled": self.reports_enabled,
                    "test_type_selection_enabled": self.test_type_selection_enabled,
                    "test_file_selection_enabled": self.test_file_selection_enabled
                }
            }
        except Exception:
            self.handle_exception("Failed to read server configuration")
            return {"status": 500}

    def handle_request(self, request, response):
        method = request.method
        uri_parts = self.parse_uri(request)

        result = None
        # /api/<function>
        if len(uri_parts) == 2:
            function = uri_parts[1]
            if method == "GET":
                if function == "status":
                    result = self.read_status()

        if result is None:
            response.status = 404
            return

        format = None
        if "format" in result:
            format = result["format"]
            if format == "application/json":
                data = None
                if "data" in result:
                    data = result["data"]
                status = 200
                if "status" in result:
                    status = result["status"]
                self.send_json(data, response, status)
                return

        status = 404
        if "status" in result:
            status = result["status"]
        response.status = status
