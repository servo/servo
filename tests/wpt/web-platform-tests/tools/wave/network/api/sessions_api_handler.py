import json
import threading

from .api_handler import ApiHandler

from ...utils.serializer import serialize_session
from ...data.exceptions.not_found_exception import NotFoundException
from ...data.exceptions.invalid_data_exception import InvalidDataException
from ...data.http_polling_client import HttpPollingClient

TOKEN_LENGTH = 36


class SessionsApiHandler(ApiHandler):
    def __init__(self, sessions_manager, results_manager, event_dispatcher, web_root):
        super(SessionsApiHandler, self).__init__(web_root)
        self._sessions_manager = sessions_manager
        self._results_manager = results_manager
        self._event_dispatcher = event_dispatcher

    def create_session(self, request, response):
        try:
            config = {}
            body = request.body.decode("utf-8")
            if body != "":
                config = json.loads(body)
            tests = {}
            if "tests" in config:
                tests = config["tests"]
            types = None
            if "types" in config:
                types = config["types"]
            timeouts = {}
            if "timeouts" in config:
                timeouts = config["timeouts"]
            reference_tokens = []
            if "reference_tokens" in config:
                reference_tokens = config["reference_tokens"]
            webhook_urls = []
            if "webhook_urls" in config:
                webhook_urls = config["webhook_urls"]
            user_agent = request.headers[b"user-agent"].decode("utf-8")
            labels = []
            if "labels" in config:
                labels = config["labels"]
            expiration_date = None
            if "expiration_date" in config:
                expiration_date = config["expiration_date"]

            session = self._sessions_manager.create_session(
                tests,
                types,
                timeouts,
                reference_tokens,
                webhook_urls,
                user_agent,
                labels,
                expiration_date
            )

            self.send_json({"token": session.token}, response)
        except InvalidDataException:
            self.handle_exception("Failed to create session")
            self.send_json({"error": "Invalid input data!"}, response, 400)

        except Exception:
            self.handle_exception("Failed to create session")
            response.status = 500

    def read_session(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]

            session = self._sessions_manager.read_session(token)
            if session is None:
                response.status = 404
                return

            data = serialize_session(session)

            del data["pending_tests"]
            del data["running_tests"]
            del data["malfunctioning_tests"]
            del data["test_state"]
            del data["date_started"]
            del data["date_finished"]
            del data["status"]

            self.send_json(data, response)
        except Exception:
            self.handle_exception("Failed to read session")
            response.status = 500

    def read_session_status(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]

            session = self._sessions_manager.read_session_status(token)
            if session is None:
                response.status = 404
                return
            data = serialize_session(session)

            del data["tests"]
            del data["pending_tests"]
            del data["running_tests"]
            del data["malfunctioning_tests"]
            del data["types"]
            del data["test_state"]
            del data["last_completed_test"]
            del data["user_agent"]
            del data["timeouts"]
            del data["browser"]
            del data["is_public"]
            del data["reference_tokens"]
            del data["webhook_urls"]

            self.send_json(data, response)
        except Exception:
            self.handle_exception("Failed to read session status")
            response.status = 500

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
            types = None
            if "types" in config:
                types = config["types"]
            timeouts = {}
            if "timeouts" in config:
                timeouts = config["timeouts"]
            reference_tokens = []
            if "reference_tokens" in config:
                reference_tokens = config["reference_tokens"]
            webhook_urls = []
            if "webhook_urls" in config:
                webhook_urls = config["webhook_urls"]

            self._sessions_manager.update_session_configuration(
                token,
                tests,
                types,
                timeouts,
                reference_tokens,
                webhook_urls
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

            event = threading.Event()
            http_polling_client = HttpPollingClient(token, event)
            self._event_dispatcher.add_session_client(http_polling_client)

            event.wait()

            message = http_polling_client.message
            self.send_json(data=message, response=response)
        except Exception:
            self.handle_exception("Failed to register event listener")
            response.status = 500

    def handle_request(self, request, response):
        method = request.method
        uri_parts = self.parse_uri(request)

        # /api/sessions
        if len(uri_parts) == 2:
            if method == "POST":
                self.create_session(request, response)
                return

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
                self.read_session(request, response)
                return
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
                    self.read_session_status(request, response)
                    return
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
            if method == "PUT":
                if function == "labels":
                    self.update_labels(request, response)
                    return

        response.status = 404
