# mypy: allow-untyped-defs

import re
from threading import Timer

from .event_dispatcher import TEST_COMPLETED_EVENT

from ..data.exceptions.not_found_exception import NotFoundException
from ..data.session import COMPLETED, ABORTED


class TestsManager:
    def initialize(
        self,
        test_loader,
        sessions_manager,
        results_manager,
        event_dispatcher
    ):
        self._test_loader = test_loader
        self._sessions_manager = sessions_manager
        self._results_manager = results_manager
        self._event_dispatcher = event_dispatcher

        self._timeouts = []

    def next_test(self, session):
        if session.status == COMPLETED or session.status == ABORTED:
            return None

        pending_tests = session.pending_tests
        running_tests = session.running_tests
        token = session.token

        if pending_tests is None:
            pending_tests = self.load_tests(session)
            session.pending_tests = pending_tests
            self._sessions_manager.update_session(session)

        if running_tests is None:
            running_tests = {}

        test = self._get_next_test_from_list(pending_tests)
        if test is None:
            return None

        pending_tests = self.remove_test_from_list(pending_tests, test)
        running_tests = self.add_test_to_list(running_tests, test)

        test_timeout = self.get_test_timeout(test, session) / 1000.0

        def handler(self, token, test):
            self._on_test_timeout(token, test)

        timer = Timer(test_timeout, handler, [self, token, test])
        self._timeouts.append({
            "test": test,
            "timeout": timer
        })

        session.pending_tests = pending_tests
        session.running_tests = running_tests
        self._sessions_manager.update_session(session)

        timer.start()
        return test

    def read_last_completed_tests(self, token, count):
        results = self._results_manager.read_results(token)

        results_tests = {}
        for api in list(results.keys()):
            results_tests[api] = []
            for result in results[api]:
                results_tests[api].append(result["test"])

        sorted_results_tests = self._sort_tests_by_execution(results_tests)
        sorted_results_tests.reverse()

        tests = {"pass": [], "fail": [], "timeout": []}

        for test in sorted_results_tests:
            api = None
            for part in test.split("/"):
                if part != "":
                    api = part
                    break

            result = None
            for potential_result in results[api]:
                if potential_result["test"] == test:
                    result = potential_result
                    break
            if result is None:
                break

            if result["status"] == "ERROR":
                if len(tests["fail"]) < count:
                    tests["fail"].append(result["test"])
            elif result["status"] == "TIMEOUT":
                if len(tests["timeout"]) < count:
                    tests["timeout"].append(result["test"])
            passes = True
            for test in result["subtests"]:
                if test["status"] != "PASS":
                    passes = False
                    break

            if passes and len(tests["pass"]) < count:
                tests["pass"].append(result["test"])
            if not passes and len(tests["fail"]) < count:
                tests["fail"].append(result["test"])
            if len(tests["pass"]) == count and len(tests["fail"]) == count \
               and len(tests["timeout"]) == count:
                return tests
        return tests

    def _sort_tests_by_execution(self, tests):
        sorted_tests = []

        for api in list(tests.keys()):
            for test in tests[api]:
                sorted_tests.append(test)

        class compare:
            def __init__(self, tests_manager, test):
                self.test = test
                self.tests_manager = tests_manager

            def __lt__(self, test_b):
                test_a = self.test
                test_b = test_b.test
                micro_test_list = {}
                api_a = ""
                for part in test_a.split("/"):
                    if part != "":
                        api_a = part
                        break
                api_b = ""
                for part in test_b.split("/"):
                    if part != "":
                        api_b = part
                        break
                if api_a == api_b:
                    micro_test_list[api_a] = [test_a, test_b]
                else:
                    micro_test_list[api_a] = [test_a]
                    micro_test_list[api_b] = [test_b]
                next_test = self.tests_manager._get_next_test_from_list(micro_test_list)
                return next_test == test_b

        sorted_tests.sort(key=lambda test: compare(self, test))
        return sorted_tests

    def _get_next_test_from_list(self, tests):
        test = None
        api = None
        has_http = True
        has_manual = True
        current_api = 0
        current_test = 0

        apis = list(tests.keys())
        apis.sort(key=lambda api: api.lower())

        for api in apis:
            tests[api].sort(key=lambda test: test.replace("/", "").lower())

        while test is None:
            if len(apis) <= current_api:
                return None
            api = apis[current_api]

            if len(tests[api]) <= current_test:
                current_api = current_api + 1
                current_test = 0

                if current_api == len(apis):
                    if has_http:
                        has_http = False
                        current_api = 0
                        test = None
                        continue

                    if has_manual:
                        has_manual = False
                        current_api = 0
                        test = None
                        has_http = True
                        continue

                    return None

                test = None
                continue
            test = tests[api][current_test]

            if "manual" in test and "https" not in test:
                return test

            if "manual" in test and "https" in test:
                if not has_http:
                    return test

            if "manual" not in test and "https" not in test:
                if not has_manual:
                    return test

            if "manual" not in test and "https" in test:
                if not has_manual and not has_http:
                    return test

            current_test = current_test + 1
            test = None

        return test

    def skip_to(self, test_list, test):
        sorted_tests = self._sort_tests_by_execution(test_list)
        if test not in sorted_tests:
            return test_list
        index = sorted_tests.index(test)
        remaining_tests = sorted_tests[index + 1:]
        remaining_tests_by_api = {}
        current_api = "___"
        for test in remaining_tests:
            if not test.startswith("/" + current_api) and \
               not test.startswith(current_api):
                current_api = next((p for p in test.split("/") if p != ""),
                                   None)
                if current_api not in remaining_tests_by_api:
                    remaining_tests_by_api[current_api] = []
            remaining_tests_by_api[current_api].append(test)
        return remaining_tests_by_api

    def remove_test_from_list(self, test_list, test):
        api = None
        for part in test.split("/"):
            if part is None or part == "":
                continue
            api = part
            break
        if api not in test_list:
            return test_list
        if test not in test_list[api]:
            return test_list
        test_list[api].remove(test)
        if len(test_list[api]) == 0:
            del test_list[api]

        return test_list

    def add_test_to_list(self, test_list, test):
        api = None
        for part in test.split("/"):
            if part is None or part == "":
                continue
            api = part
            break
        if api in test_list and test in test_list[api]:
            return test_list
        if api not in test_list:
            test_list[api] = []
        test_list[api].append(test)
        return test_list

    def get_test_timeout(self, test, session):
        timeouts = session.timeouts
        test_timeout = None

        for path in list(timeouts.keys()):
            pattern = re.compile("^" + path.replace(".", ""))
            if pattern.match(test.replace(".", "")) is not None:
                test_timeout = timeouts[path]
                break

        if test_timeout is None:
            if "manual" in test:
                test_timeout = timeouts["manual"]
            else:
                test_timeout = timeouts["automatic"]

        return test_timeout

    def _on_test_timeout(self, token, test):
        data = {
            "test": test,
            "status": "TIMEOUT",
            "message": None,
            "subtests": [
                {
                    "status": "TIMEOUT",
                    "xstatus": "SERVERTIMEOUT"
                }
            ]
        }

        self._results_manager.create_result(token, data)

    def read_tests(self):
        return self._test_loader.get_tests()

    def complete_test(self, test, session):
        running_tests = session.running_tests

        running_tests = self.remove_test_from_list(running_tests, test)
        session.running_tests = running_tests

        timeout = next((t for t in self._timeouts if t["test"] == test), None)
        timeout["timeout"].cancel()
        self._timeouts.remove(timeout)

        self.update_tests(
            running_tests=running_tests,
            session=session
        )

        self._event_dispatcher.dispatch_event(
            dispatcher_token=session.token,
            event_type=TEST_COMPLETED_EVENT,
            data=test
        )

    def update_tests(
        self,
        pending_tests=None,
        running_tests=None,
        session=None
    ):
        if pending_tests is not None:
            session.pending_tests = pending_tests

        if running_tests is not None:
            session.running_tests = running_tests

        self._sessions_manager.update_session(session)

    def calculate_test_files_count(self, tests):
        count = {}
        for api in tests:
            count[api] = len(tests[api])
        return count

    def read_malfunctioning_tests(self, token):
        session = self._sessions_manager.read_session(token)
        return session.malfunctioning_tests

    def update_malfunctioning_tests(self, token, tests):
        if token is None:
            return
        if tests is None:
            return

        session = self._sessions_manager.read_session(token)
        if session is None:
            raise NotFoundException("Could not find session using token: " + token)
        if session.is_public:
            return
        session.malfunctioning_tests = tests
        self._sessions_manager.update_session(session)

    def load_tests(self, session):
        pending_tests = self._test_loader.get_tests(
            session.test_types,
            include_list=session.tests["include"],
            exclude_list=session.tests["exclude"],
            reference_tokens=session.reference_tokens
        )

        last_completed_test = session.last_completed_test
        if last_completed_test is not None:
            pending_tests = self.skip_to(pending_tests, last_completed_test)

        return pending_tests
