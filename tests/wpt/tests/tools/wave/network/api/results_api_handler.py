# mypy: allow-untyped-defs

import json

from .api_handler import ApiHandler
from ...data.exceptions.duplicate_exception import DuplicateException
from ...data.exceptions.invalid_data_exception import InvalidDataException


class ResultsApiHandler(ApiHandler):
    def __init__(self, results_manager, session_manager, web_root):
        super().__init__(web_root)
        self._results_manager = results_manager
        self._sessions_manager = session_manager

    def create_result(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]

            data = None
            body = request.body.decode("utf-8")
            if body != "":
                data = json.loads(body)

            self._results_manager.create_result(token, data)

        except Exception:
            self.handle_exception("Failed to create result")
            response.status = 500

    def read_results(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]

            session = self._sessions_manager.read_session(token)
            if session is None:
                response.status = 404
                return

            results = self._results_manager.read_results(token)

            self.send_json(response=response, data=results)

        except Exception:
            self.handle_exception("Failed to read results")
            response.status = 500

    def read_results_compact(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]

            results = self._results_manager.read_flattened_results(token)

            self.send_json(response=response, data=results)

        except Exception:
            self.handle_exception("Failed to read compact results")
            response.status = 500

    def read_results_api_wpt_report_url(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]
            api = uri_parts[3]

            uri = self._results_manager.read_results_wpt_report_uri(token, api)
            self.send_json({"uri": uri}, response)
        except Exception:
            self.handle_exception("Failed to read results report url")
            response.status = 500

    def read_results_api_wpt_multi_report_uri(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            api = uri_parts[2]
            query = self.parse_query_parameters(request)
            tokens = query["tokens"].split(",")
            uri = self._results_manager.read_results_wpt_multi_report_uri(
                tokens,
                api
            )
            self.send_json({"uri": uri}, response)
        except Exception:
            self.handle_exception("Failed to read results multi report url")
            response.status = 500

    def download_results_api_json(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]
            api = uri_parts[3]
            blob = self._results_manager.export_results_api_json(token, api)
            if blob is None:
                response.status = 404
                return
            file_path = self._results_manager.get_json_path(token, api)
            file_name = "{}-{}-{}".format(
                token.split("-")[0],
                api,
                file_path.split("/")[-1]
            )
            self.send_zip(blob, file_name, response)
        except Exception:
            self.handle_exception("Failed to download api json")
            response.status = 500

    def import_results_api_json(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]
            api = uri_parts[3]
            blob = request.body

            self._results_manager.import_results_api_json(token, api, blob)

            response.status = 200
        except Exception:
            self.handle_exception("Failed to upload api json")
            response.status = 500

    def download_results_all_api_jsons(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]
            blob = self._results_manager.export_results_all_api_jsons(token)
            file_name = token.split("-")[0] + "_results_json.zip"
            self.send_zip(blob, file_name, response)
        except Exception:
            self.handle_exception("Failed to download all api jsons")
            response.status = 500

    def download_results(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]
            blob = self._results_manager.export_results(token)
            if blob is None:
                response.status = 404
                return
            file_name = token + ".zip"
            self.send_zip(blob, file_name, response)
        except Exception:
            self.handle_exception("Failed to download results")
            response.status = 500

    def download_results_overview(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]
            blob = self._results_manager.export_results_overview(token)
            if blob is None:
                response.status = 404
                return
            file_name = token.split("-")[0] + "_results_html.zip"
            self.send_zip(blob, file_name, response)
        except Exception:
            self.handle_exception("Failed to download results overview")
            response.status = 500

    def import_results(self, request, response):
        try:
            blob = request.body
            token = self._results_manager.import_results(blob)
            self.send_json({"token": token}, response)
        except DuplicateException:
            self.handle_exception("Failed to import results")
            self.send_json({"error": "Session already exists!"}, response, 400)
            return
        except InvalidDataException:
            self.handle_exception("Failed to import results")
            self.send_json({"error": "Invalid input data!"}, response, 400)
            return
        except Exception:
            self.handle_exception("Failed to import results")
            response.status = 500

    def handle_request(self, request, response):
        method = request.method
        uri_parts = self.parse_uri(request)

        # /api/results/<token>
        if len(uri_parts) == 3:
            if method == "POST":
                if uri_parts[2] == "import":
                    self.import_results(request, response)
                    return
                self.create_result(request, response)
                return

            if method == "GET":
                self.read_results(request, response)
                return

        # /api/results/<token>/<function>
        if len(uri_parts) == 4:
            function = uri_parts[3]
            if method == "GET":
                if function == "compact":
                    self.read_results_compact(request, response)
                    return
                if function == "reporturl":
                    return self.read_results_api_wpt_multi_report_uri(request,
                                                                      response)
                if function == "json":
                    self.download_results_all_api_jsons(request, response)
                    return
                if function == "export":
                    self.download_results(request, response)
                    return
                if function == "overview":
                    self.download_results_overview(request, response)
                    return

        # /api/results/<token>/<api>/<function>
        if len(uri_parts) == 5:
            function = uri_parts[4]
            if method == "GET":
                if function == "reporturl":
                    self.read_results_api_wpt_report_url(request, response)
                    return
                if function == "json":
                    self.download_results_api_json(request, response)
                    return
            if method == "POST":
                if function == "json":
                    self.import_results_api_json(request, response)
                    return

        response.status = 404
