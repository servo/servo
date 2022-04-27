import uuid
import time
import os
import json
import re

from threading import Timer

from .test_loader import AUTOMATIC, MANUAL
from ..data.session import Session, PENDING, PAUSED, RUNNING, ABORTED, COMPLETED
from ..utils.user_agent_parser import parse_user_agent
from .event_dispatcher import STATUS_EVENT, RESUME_EVENT
from ..data.exceptions.not_found_exception import NotFoundException
from ..data.exceptions.invalid_data_exception import InvalidDataException
from ..utils.deserializer import deserialize_session

DEFAULT_TEST_TYPES = [AUTOMATIC, MANUAL]
DEFAULT_TEST_PATHS = ["/"]
DEFAULT_TEST_AUTOMATIC_TIMEOUT = 60000
DEFAULT_TEST_MANUAL_TIMEOUT = 300000


class SessionsManager:
    def initialize(self,
                   test_loader,
                   event_dispatcher,
                   tests_manager,
                   results_directory,
                   results_manager,
                   configuration):
        self._test_loader = test_loader
        self._sessions = {}
        self._expiration_timeout = None
        self._event_dispatcher = event_dispatcher
        self._tests_manager = tests_manager
        self._results_directory = results_directory
        self._results_manager = results_manager
        self._configuration = configuration

    def create_session(
        self,
        tests=None,
        test_types=None,
        timeouts=None,
        reference_tokens=None,
        user_agent=None,
        labels=None,
        expiration_date=None,
        type=None
    ):
        if tests is None:
            tests = {}
        if timeouts is None:
            timeouts = {}
        if reference_tokens is None:
            reference_tokens = []
        if user_agent is None:
            user_agent = ""
        if labels is None:
            labels = []

        if "include" not in tests:
            tests["include"] = DEFAULT_TEST_PATHS
        if "exclude" not in tests:
            tests["exclude"] = []
        if "automatic" not in timeouts:
            timeouts["automatic"] = self._configuration["timeouts"]["automatic"]
        if "manual" not in timeouts:
            timeouts["manual"] = self._configuration["timeouts"]["manual"]
        if test_types is None:
            test_types = DEFAULT_TEST_TYPES

        for test_type in test_types:
            if test_type != "automatic" and test_type != "manual":
                raise InvalidDataException(f"Unknown type '{test_type}'")

        token = str(uuid.uuid1())
        pending_tests = self._test_loader.get_tests(
            test_types,
            include_list=tests["include"],
            exclude_list=tests["exclude"],
            reference_tokens=reference_tokens)

        browser = parse_user_agent(user_agent)

        test_files_count = self._tests_manager.calculate_test_files_count(
            pending_tests
        )

        test_state = {}
        for api in test_files_count:
            test_state[api] = {
                "pass": 0,
                "fail": 0,
                "timeout": 0,
                "not_run": 0,
                "total": test_files_count[api],
                "complete": 0}

        date_created = int(time.time() * 1000)

        session = Session(
            token=token,
            tests=tests,
            user_agent=user_agent,
            browser=browser,
            test_types=test_types,
            timeouts=timeouts,
            pending_tests=pending_tests,
            running_tests={},
            test_state=test_state,
            status=PENDING,
            reference_tokens=reference_tokens,
            labels=labels,
            type=type,
            expiration_date=expiration_date,
            date_created=date_created
        )

        self._push_to_cache(session)
        if expiration_date is not None:
            self._set_expiration_timer()

        return session

    def read_session(self, token):
        if token is None:
            return None
        session = self._read_from_cache(token)
        if session is None or session.test_state is None:
            print("loading session from file system")
            session = self.load_session(token)
        if session is not None:
            self._push_to_cache(session)
        return session

    def read_sessions(self, index=None, count=None):
        if index is None:
            index = 0
        if count is None:
            count = 10
        self.load_all_sessions_info()
        sessions = []
        for it_index, token in enumerate(self._sessions):
            if it_index < index:
                continue
            if len(sessions) == count:
                break
            sessions.append(token)
        return sessions

    def read_session_status(self, token):
        if token is None:
            return None
        session = self._read_from_cache(token)
        if session is None:
            session = self.load_session_info(token)
            if session is None:
                return None
        if session.test_state is None:
            session = self.load_session(token)
        if session is not None:
            self._push_to_cache(session)
        return session

    def read_public_sessions(self):
        self.load_all_sessions_info()
        session_tokens = []
        for token in self._sessions:
            session = self._sessions[token]
            if not session.is_public:
                continue
            session_tokens.append(token)

        return session_tokens

    def update_session(self, session):
        self._push_to_cache(session)

    def update_session_configuration(
        self, token, tests, test_types, timeouts, reference_tokens, type
    ):
        session = self.read_session(token)
        if session is None:
            raise NotFoundException("Could not find session")
        if session.status != PENDING:
            return

        if tests is not None:
            if "include" not in tests:
                tests["include"] = session.tests["include"]
            if "exclude" not in tests:
                tests["exclude"] = session.tests["exclude"]
            if reference_tokens is None:
                reference_tokens = session.reference_tokens
            if test_types is None:
                test_types = session.test_types

            pending_tests = self._test_loader.get_tests(
                include_list=tests["include"],
                exclude_list=tests["exclude"],
                reference_tokens=reference_tokens,
                test_types=test_types
            )
            session.pending_tests = pending_tests
            session.tests = tests
            test_files_count = self._tests_manager.calculate_test_files_count(
                pending_tests)
            test_state = {}
            for api in test_files_count:
                test_state[api] = {
                    "pass": 0,
                    "fail": 0,
                    "timeout": 0,
                    "not_run": 0,
                    "total": test_files_count[api],
                    "complete": 0,
                }
            session.test_state = test_state

        if test_types is not None:
            session.test_types = test_types
        if timeouts is not None:
            if AUTOMATIC not in timeouts:
                timeouts[AUTOMATIC] = session.timeouts[AUTOMATIC]
            if MANUAL not in timeouts:
                timeouts[MANUAL] = session.timeouts[MANUAL]
            session.timeouts = timeouts
        if reference_tokens is not None:
            session.reference_tokens = reference_tokens
        if type is not None:
            session.type = type

        self._push_to_cache(session)
        return session

    def update_labels(self, token, labels):
        if token is None or labels is None:
            return
        session = self.read_session(token)
        if session is None:
            return
        if session.is_public:
            return
        session.labels = labels
        self._push_to_cache(session)

    def delete_session(self, token):
        session = self.read_session(token)
        if session is None:
            return
        if session.is_public is True:
            return
        del self._sessions[token]

    def add_session(self, session):
        if session is None:
            return
        self._push_to_cache(session)

    def load_all_sessions(self):
        if not os.path.isdir(self._results_directory):
            return
        tokens = os.listdir(self._results_directory)
        for token in tokens:
            self.load_session(token)

    def load_all_sessions_info(self):
        if not os.path.isdir(self._results_directory):
            return
        tokens = os.listdir(self._results_directory)
        for token in tokens:
            if token in self._sessions:
                continue
            self.load_session_info(token)

    def load_session(self, token):
        session = self.load_session_info(token)
        if session is None:
            return None

        if session.test_state is None:
            results = self._results_manager.load_results(token)
            test_state = self._results_manager.parse_test_state(results)
            session.test_state = test_state
            self._results_manager.create_info_file(session)

        self._push_to_cache(session)
        return session

    def load_session_info(self, token):
        result_directory = os.path.join(self._results_directory, token)
        if not os.path.isdir(result_directory):
            return None
        info_file = os.path.join(result_directory, "info.json")
        if not os.path.isfile(info_file):
            return None

        info_data = None
        with open(info_file) as file:
            info_data = file.read()
        parsed_info_data = json.loads(info_data)

        session = deserialize_session(parsed_info_data)
        self._push_to_cache(session)
        return session

    def _push_to_cache(self, session):
        self._sessions[session.token] = session

    def _read_from_cache(self, token):
        if token not in self._sessions:
            return None
        return self._sessions[token]

    def _set_expiration_timer(self):
        expiring_sessions = self._read_expiring_sessions()
        if len(expiring_sessions) == 0:
            return

        next_session = expiring_sessions[0]
        for session in expiring_sessions:
            if next_session.expiration_date > session.expiration_date:
                next_session = session

        if self._expiration_timeout is not None:
            self._expiration_timeout.cancel()

        timeout = next_session.expiration_date / 1000 - time.time()
        if timeout < 0:
            timeout = 0

        def handle_timeout(self):
            self._delete_expired_sessions()
            self._set_expiration_timer()

        self._expiration_timeout = Timer(timeout, handle_timeout, [self])
        self._expiration_timeout.start()

    def _delete_expired_sessions(self):
        expiring_sessions = self._read_expiring_sessions()
        now = int(time.time() * 1000)

        for session in expiring_sessions:
            if session.expiration_date < now:
                self.delete_session(session.token)

    def _read_expiring_sessions(self):
        expiring_sessions = []
        for token in self._sessions:
            session = self._sessions[token]
            if session.expiration_date is None:
                continue
            expiring_sessions.append(session)
        return expiring_sessions

    def start_session(self, token):
        session = self.read_session(token)

        if session is None:
            return

        if session.status != PENDING and session.status != PAUSED:
            return

        if session.status == PENDING:
            session.date_started = int(time.time() * 1000)
            session.expiration_date = None

        session.status = RUNNING
        self.update_session(session)

        self._event_dispatcher.dispatch_event(
            token,
            event_type=STATUS_EVENT,
            data=session.status
        )

    def pause_session(self, token):
        session = self.read_session(token)
        if session.status != RUNNING:
            return
        session.status = PAUSED
        self.update_session(session)
        self._event_dispatcher.dispatch_event(
            token,
            event_type=STATUS_EVENT,
            data=session.status
        )
        self._results_manager.persist_session(session)

    def stop_session(self, token):
        session = self.read_session(token)
        if session.status == ABORTED or session.status == COMPLETED:
            return
        session.status = ABORTED
        session.date_finished = int(time.time() * 1000)
        self.update_session(session)
        self._event_dispatcher.dispatch_event(
            token,
            event_type=STATUS_EVENT,
            data=session.status
        )

    def resume_session(self, token, resume_token):
        session = self.read_session(token)
        if session.status != PENDING:
            return
        self._event_dispatcher.dispatch_event(
            token,
            event_type=RESUME_EVENT,
            data=resume_token
        )
        self.delete_session(token)

    def complete_session(self, token):
        session = self.read_session(token)
        if session.status == COMPLETED or session.status == ABORTED:
            return
        session.status = COMPLETED
        session.date_finished = int(time.time() * 1000)
        self.update_session(session)
        self._event_dispatcher.dispatch_event(
            token,
            event_type=STATUS_EVENT,
            data=session.status
        )

    def test_in_session(self, test, session):
        return self._test_list_contains_test(test, session.pending_tests) \
            or self._test_list_contains_test(test, session.running_tests)

    def is_test_complete(self, test, session):
        return not self._test_list_contains_test(test, session.pending_tests) \
            and not self._test_list_contains_test(test, session.running_tests)

    def is_test_running(self, test, session):
        return self._test_list_contains_test(test, session.running_tests)

    def _test_list_contains_test(self, test, test_list):
        for api in list(test_list.keys()):
            if test in test_list[api]:
                return True
        return False

    def is_api_complete(self, api, session):
        return api not in session.pending_tests \
            and api not in session.running_tests

    def get_test_path_with_query(self, test, session):
        query_string = ""
        include_list = session.tests["include"]
        for include_test in include_list:
            split = include_test.split("?")
            query = ""
            if len(split) > 1:
                include_test = split[0]
                query = split[1]
            pattern = re.compile("^" + include_test)
            if pattern.match(test) is not None:
                query_string += query + "&"
        return f"{test}?{query_string}"

    def find_token(self, fragment):
        if len(fragment) < 8:
            return None
        tokens = []
        for token in self._sessions:
            if token.startswith(fragment):
                tokens.append(token)
        if len(tokens) != 1:
            return None
        return tokens[0]

    def get_total_sessions(self):
        return len(self._sessions)
