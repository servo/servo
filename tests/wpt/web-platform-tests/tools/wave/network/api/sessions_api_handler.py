import json
import threading

from .api_handler import ApiHandler

from ...utils.serializer import serialize_session
from ...data.exceptions.not_found_exception import NotFoundException
from ...data.exceptions.invalid_data_exception import InvalidDataException
from ...data.http_polling_event_listener import HttpPollingEventListener

TOKEN_LENGTH = 36


class SessionsApiHandler(ApiHandler):
    def __init__(
        self,
        sessions_manager,
        results_manager,
        event_dispatcher,
        web_root,
        read_sessions_enabled
    ):
        super().__init__(web_root)
        self._sessions_manager = sessions_manager
        self._results_manager = results_manager
        self._event_dispatcher = event_dispatcher
        self._read_sessions_enabled = read_sessions_enabled

    def create_session(self, body, headers):
        try:
            config = {}
            body = body.decode("utf-8")
            if body != "":
                config = json.loads(body)
            tests = {}
            if "tests" in config:
                tests = config["tests"]
            test_types = None
            if "types" in config:
                test_types = config["types"]
            timeouts = {}
            if "timeouts" in config:
                timeouts = config["timeouts"]
            reference_tokens = []
            if "reference_tokens" in config:
                reference_tokens = config["reference_tokens"]
            user_agent = headers[b"user-agent"].decode("utf-8")
            labels = []
            if "labels" in config:
                labels = config["labels"]
            expiration_date = None
            if "expiration_date" in config:
                expiration_date = config["expiration_date"]
            type = None
            if "type" in config:
                type = config["type"]

            session = self._sessions_manager.create_session(
                tests,
                test_types,
                timeouts,
                reference_tokens,
                user_agent,
                labels,
                expiration_date,
                type
            )

            return {
                "format": "application/json",
                "data": {"token": session.token}
            }

        except InvalidDataException:
            self.handle_exception("Failed to create session")
            return {
                "format": "application/json",
                "data": {"error": "Invalid input data!"},
                "status": 400
            }

        except Exception:
            self.handle_exception("Failed to create session")
            return {"status": 500}

    def read_session(self, token):
        try:

            session = self._sessions_manager.read_session(token)
            if session is None:
                return {"status": 404}

            data = serialize_session(session)

            return {
                "format": "application/json",
                "data": {
                    "token": data["token"],
                    "tests": data["tests"],
                    "types": data["types"],
                    "timeouts": data["timeouts"],
                    "reference_tokens": data["reference_tokens"],
                    "user_agent": data["user_agent"],
                    "browser": data["browser"],
                    "is_public": data["is_public"],
                    "date_created": data["date_created"],
                    "labels": data["labels"]
                }
            }
        except Exception:
            self.handle_exception("Failed to read session")
            return {"status": 500}

    def read_sessions(self, query_parameters, uri_path):
        try:
            index = 0
            if "index" in query_parameters:
                index = int(query_parameters["index"])
            count = 10
            if "count" in query_parameters:
                count = int(query_parameters["count"])
            expand = []
            if "expand" in query_parameters:
                expand = query_parameters["expand"].split(",")

            session_tokens = self._sessions_manager.read_sessions(index=index, count=count)
            total_sessions = self._sessions_manager.get_total_sessions()

            embedded = {}

            for relation in expand:
                if relation == "configuration":
                    configurations = []
                    for token in session_tokens:
                        result = self.read_session(token)
                        if "status" in result and result["status"] != 200:
                            continue
                        configurations.append(result["data"])
                    embedded["configuration"] = configurations

                if relation == "status":
                    statuses = []
                    for token in session_tokens:
                        result = self.read_session_status(token)
                        if "status" in result and result["status"] != 200:
                            continue
                        statuses.append(result["data"])
                    embedded["status"] = statuses

            uris = {
                "self": uri_path,
                "configuration": self._web_root + "api/sessions/{token}",
                "status": self._web_root + "api/sessions/{token}/status"
            }

            data = self.create_hal_list(session_tokens, uris, index, count, total=total_sessions)

            if len(embedded) > 0:
                data["_embedded"] = embedded

            return {
                "format": "application/json",
                "data": data
            }
        except Exception:
            self.handle_exception("Failed to read session")
            return {"status": 500}

    def read_session_status(self, token):
        try:
            session = self._sessions_manager.read_session_status(token)
            if session is None:
                return {"status": 404}

            data = serialize_session(session)

            return {
                "format": "application/json",
                "data": {
                    "token": data["token"],
                    "status": data["status"],
                    "date_started": data["date_started"],
                    "date_finished": data["date_finished"],
                    "expiration_date": data["expiration_date"]
                }
            }
        except Exception:
            self.handle_exception("Failed to read session status")
            return {"status": 500}

    def read_public_sessions(self, request, response):
        try:
            session_tokens = self._sessions_manager.read_public_sessions()

            self.send_json(session_tokens, response)
        except Exception:
            self.handle_exception("Failed to read public sessions")
            response.status = 500

    def update_session_configuration(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]

            config = {}
            body = request.body.decode("utf-8")
            if body != "":
                config = json.loads(body)

            tests = {}
            if "tests" in config:
                tests = config["tests"]
            test_types = None
            if "types" in config:
                test_types = config["types"]
            timeouts = {}
            if "timeouts" in config:
                timeouts = config["timeouts"]
            reference_tokens = []
            if "reference_tokens" in config:
                reference_tokens = config["reference_tokens"]
            type = None
            if "type" in config:
                type = config["type"]

            self._sessions_manager.update_session_configuration(
                token,
                tests,
                test_types,
                timeouts,
                reference_tokens,
                type
            )
        except NotFoundException:
            self.handle_exception("Failed to update session configuration")
            response.status = 404
        except Exception:
            self.handle_exception("Failed to update session configuration")
            response.status = 500

    def update_labels(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            # convert unicode to ascii to get a text type, ignore special chars
            token = uri_parts[2]
            body = request.body.decode("utf-8")
            labels = None
            if body != "":
                labels = json.loads(body)
                if "labels" in labels:
                    labels = labels["labels"]

            self._sessions_manager.update_labels(token=token, labels=labels)
        except Exception:
            self.handle_exception("Failed to update labels")
            response.status = 500

    def delete_session(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]

            session = self._sessions_manager.read_session(token)
            if session is None:
                response.status = 404
                return

            self._sessions_manager.delete_session(token)
            self._results_manager.delete_results(token)
        except Exception:
            self.handle_exception("Failed to delete session")
            response.status = 500

    def start_session(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]

            self._sessions_manager.start_session(token)
        except Exception:
            self.handle_exception("Failed to start session")
            response.status = 500

    def pause_session(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]

            self._sessions_manager.pause_session(token)
        except Exception:
            self.handle_exception("Failed to pause session")
            response.status = 500

    def stop_session(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]

            self._sessions_manager.stop_session(token)
        except Exception:
            self.handle_exception("Failed to stop session")
            response.status = 500

    def resume_session(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]

            resume_token = None
            body = request.body.decode("utf-8")
            if body != "":
                resume_token = json.loads(body)["resume_token"]

            self._sessions_manager.resume_session(token, resume_token)
        except Exception:
            self.handle_exception("Failed to resume session")
            response.status = 500

    def find_session(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            fragment = uri_parts[2]
            token = self._sessions_manager.find_token(fragment)
            if token is None:
                response.status = 404
                return
            self.send_json({"token": token}, response)
        except Exception:
            self.handle_exception("Failed to find session")
            response.status = 500

    def register_event_listener(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]

            query_parameters = self.parse_query_parameters(request)
            last_event_number = None
            if ("last_event" in query_parameters):
                last_event_number = int(query_parameters["last_event"])

            event = threading.Event()
            http_polling_event_listener = HttpPollingEventListener(token, event)
            event_listener_token = self._event_dispatcher.add_event_listener(http_polling_event_listener, last_event_number)

            event.wait()

            message = http_polling_event_listener.message
            self.send_json(data=message, response=response)
            self._event_dispatcher.remove_event_listener(event_listener_token)
        except Exception:
            self.handle_exception("Failed to register event listener")
            response.status = 500

    def push_event(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]
            message = None
            body = request.body.decode("utf-8")
            if body != "":
                message = json.loads(body)

            self._event_dispatcher.dispatch_event(
                token,
                message["type"],
                message["data"])
        except Exception:
            self.handle_exception("Failed to push session event")

    def handle_request(self, request, response):
        method = request.method
        uri_parts = self.parse_uri(request)
        body = request.body
        headers = request.headers
        query_parameters = self.parse_query_parameters(request)
        uri_path = request.url_parts.path

        result = None
        # /api/sessions
        if len(uri_parts) == 2:
            if method == "POST":
                result = self.create_session(body, headers)
            if method == "GET":
                if self._read_sessions_enabled:
                    result = self.read_sessions(query_parameters, uri_path)

        # /api/sessions/<token>
        if len(uri_parts) == 3:
            function = uri_parts[2]
            if method == "GET":
                if function == "public":
                    self.read_public_sessions(request, response)
                    return
                if len(function) != TOKEN_LENGTH:
                    self.find_session(request, response)
                    return
                result = self.read_session(token=uri_parts[2])
            if method == "PUT":
                self.update_session_configuration(request, response)
                return
            if method == "DELETE":
                self.delete_session(request, response)
                return

        # /api/sessions/<token>/<function>
        if len(uri_parts) == 4:
            function = uri_parts[3]
            if method == "GET":
                if function == "status":
                    result = self.read_session_status(token=uri_parts[2])
                if function == "events":
                    self.register_event_listener(request, response)
                    return
            if method == "POST":
                if function == "start":
                    self.start_session(request, response)
                    return
                if function == "pause":
                    self.pause_session(request, response)
                    return
                if function == "stop":
                    self.stop_session(request, response)
                    return
                if function == "resume":
                    self.resume_session(request, response)
                    return
                if function == "events":
                    self.push_event(request, response)
                    return
            if method == "PUT":
                if function == "labels":
                    self.update_labels(request, response)
                    return

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
