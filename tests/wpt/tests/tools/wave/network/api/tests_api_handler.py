# mypy: allow-untyped-defs

import json

from urllib.parse import urlunsplit

from .api_handler import ApiHandler
from ...utils.serializer import serialize_session
from ...data.session import PAUSED, COMPLETED, ABORTED, PENDING, RUNNING

DEFAULT_LAST_COMPLETED_TESTS_COUNT = 5
DEFAULT_LAST_COMPLETED_TESTS_STATUS = ["ALL"]

EXECUTION_MODE_AUTO = "auto"
EXECUTION_MODE_MANUAL = "manual"
EXECUTION_MODE_PROGRAMMATIC = "programmatic"


class TestsApiHandler(ApiHandler):
    def __init__(
        self,
        wpt_port,
        wpt_ssl_port,
        tests_manager,
        sessions_manager,
        hostname,
        web_root,
        test_loader
    ):
        super().__init__(web_root)
        self._tests_manager = tests_manager
        self._sessions_manager = sessions_manager
        self._wpt_port = wpt_port
        self._wpt_ssl_port = wpt_ssl_port
        self._hostname = hostname
        self._web_root = web_root
        self._test_loader = test_loader

    def read_tests(self, response):
        tests = self._tests_manager.read_tests()
        self.send_json(tests, response)

    def read_session_tests(self, request, response):
        uri_parts = self.parse_uri(request)
        token = uri_parts[2]
        session = self._sessions_manager.read_session(token)

        if session is None:
            response.status = 404
            return

        data = serialize_session(session)
        tests = {
            "token": token,
            "pending_tests": data["pending_tests"],
            "running_tests": data["running_tests"]
        }
        self.send_json(tests, response)

    def read_next_test(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]

            hostname = self._hostname

            session = self._sessions_manager.read_session(token)
            if session is None:
                response.status = 404
                return

            if session.status == PAUSED:
                url = self._generate_wave_url(
                    hostname=hostname,
                    uri="pause.html",
                    token=token
                )
                self.send_json({"next_test": url}, response)
                return
            if session.status == COMPLETED or session.status == ABORTED:
                url = self._generate_wave_url(
                    hostname=hostname,
                    uri="finish.html",
                    token=token
                )
                self.send_json({"next_test": url}, response)
                return
            if session.status == PENDING:
                url = self._generate_wave_url(
                    hostname=hostname,
                    uri="newsession.html",
                    token=token
                )
                self.send_json({"next_test": url}, response)
                return

            test = self._tests_manager.next_test(session)

            if test is None:
                if session.status != RUNNING:
                    return
                url = self._generate_wave_url(
                    hostname=hostname,
                    uri="finish.html",
                    token=token
                )
                self.send_json({"next_test": url}, response)
                self._sessions_manager.complete_session(token)
                return

            test_timeout = self._tests_manager.get_test_timeout(
                test=test, session=session)

            test = self._sessions_manager.get_test_path_with_query(test, session)
            url = self._generate_test_url(
                test=test,
                token=token,
                test_timeout=test_timeout,
                hostname=hostname)

            self.send_json({
                "next_test": url
            }, response)
        except Exception:
            self.handle_exception("Failed to read next test")
            response.status = 500

    def read_last_completed(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]
            query = self.parse_query_parameters(request)
            count = None
            if "count" in query:
                count = query["count"]
            else:
                count = DEFAULT_LAST_COMPLETED_TESTS_COUNT

            status = None
            if "status" in query:
                status = query["status"].split(",")
            else:
                status = DEFAULT_LAST_COMPLETED_TESTS_STATUS

            completed_tests = self._tests_manager.read_last_completed_tests(
                token, count)
            tests = {}
            for one_status in status:
                one_status = one_status.lower()
                if one_status == "pass":
                    tests["pass"] = completed_tests["pass"]
                    continue
                if one_status == "fail":
                    tests["fail"] = completed_tests["fail"]
                    continue
                if one_status == "timeout":
                    tests["timeout"] = completed_tests["timeout"]
                    continue
                if one_status == "all":
                    tests["pass"] = completed_tests["pass"]
                    tests["fail"] = completed_tests["fail"]
                    tests["timeout"] = completed_tests["timeout"]
                    break
            self.send_json(data=tests, response=response)
        except Exception:
            self.handle_exception("Failed to read last completed tests")
            response.status = 500

    def read_malfunctioning(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]
            tm = self._tests_manager
            malfunctioning_tests = tm.read_malfunctioning_tests(token)

            self.send_json(data=malfunctioning_tests, response=response)
        except Exception:
            self.handle_exception("Failed to read malfunctioning tests")
            response.status = 500

    def update_malfunctioning(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]

            data = None
            body = request.body.decode("utf-8")
            if body != "":
                data = json.loads(body)

            self._tests_manager.update_malfunctioning_tests(token, data)
        except Exception:
            self.handle_exception("Failed to update malfunctioning tests")
            response.status = 500

    def read_available_apis(self, request, response):
        try:
            apis = self._test_loader.get_apis()
            self.send_json(apis, response)
        except Exception:
            self.handle_exception("Failed to read available APIs")
            response.status = 500

    def handle_request(self, request, response):
        method = request.method
        uri_parts = self.parse_uri(request)

        # /api/tests
        if len(uri_parts) == 2:
            if method == "GET":
                self.read_tests(response)
                return

        # /api/tests/<token>
        if len(uri_parts) == 3:
            if method == "GET":
                if uri_parts[2] == "apis":
                    self.read_available_apis(request, response)
                    return
                self.read_session_tests(request, response)
                return

        # /api/tests/<token>/<function>
        if len(uri_parts) == 4:
            function = uri_parts[3]
            if method == "GET":
                if function == "next":
                    self.read_next_test(request, response)
                    return
                if function == "last_completed":
                    self.read_last_completed(request, response)
                    return
                if function == "malfunctioning":
                    self.read_malfunctioning(request, response)
                    return
            if method == "PUT":
                if function == "malfunctioning":
                    self.update_malfunctioning(request, response)
                    return

        response.status = 404

    def _generate_wave_url(self, hostname, uri, token):
        if self._web_root is not None:
            uri = self._web_root + uri

        return self._generate_url(
            hostname=hostname,
            uri=uri,
            port=self._wpt_port,
            query="token=" + token
        )

    def _generate_test_url(self, hostname, test, token, test_timeout):
        protocol = "http"
        port = self._wpt_port

        if "https" in test:
            protocol = "https"
            port = self._wpt_ssl_port

        test_query = ""
        split = test.split("?")
        if len(split) > 1:
            test = split[0]
            test_query = split[1]

        query = "token={}&timeout={}&https_port={}&web_root={}&{}".format(
                token,
                test_timeout,
                self._wpt_ssl_port,
                self._web_root,
                test_query
        )

        return self._generate_url(
            protocol=protocol,
            hostname=hostname,
            port=port,
            uri=test,
            query=query
        )

    def _generate_url(self,
                      hostname,
                      port=None,
                      uri=None,
                      query=None,
                      protocol=None):
        if port is None:
            port = 80
        if uri is None:
            uri = "/"
        if query is None:
            query = ""
        if protocol is None:
            protocol = "http"
        return urlunsplit([protocol, f"{hostname}:{port}", uri, query, ''])
