# mypy: allow-untyped-defs

import os
import shutil
import re
import json
import hashlib
import zipfile
import time
from threading import Timer

from ..utils.user_agent_parser import parse_user_agent, abbreviate_browser_name
from ..utils.serializer import serialize_session
from ..utils.deserializer import deserialize_session
from ..data.exceptions.invalid_data_exception import InvalidDataException
from ..data.exceptions.duplicate_exception import DuplicateException
from ..data.exceptions.not_found_exception import NotFoundException
from ..data.exceptions.permission_denied_exception import PermissionDeniedException
from .wpt_report import generate_report, generate_multi_report
from ..data.session import COMPLETED

WAVE_SRC_DIR = "./tools/wave"
RESULTS_FILE_REGEX = r"^\w\w\d\d\d?\.json$"
RESULTS_FILE_PATTERN = re.compile(RESULTS_FILE_REGEX)
SESSION_RESULTS_TIMEOUT = 60*30  # 30min


class ResultsManager:
    def initialize(
        self,
        results_directory_path,
        sessions_manager,
        tests_manager,
        import_results_enabled,
        reports_enabled,
        persisting_interval
    ):
        self._results_directory_path = results_directory_path
        self._sessions_manager = sessions_manager
        self._tests_manager = tests_manager
        self._import_results_enabled = import_results_enabled
        self._reports_enabled = reports_enabled
        self._results = {}
        self._persisting_interval = persisting_interval
        self._timeouts = {}

    def create_result(self, token, data):
        result = self.prepare_result(data)
        test = result["test"]

        session = self._sessions_manager.read_session(token)

        if session is None:
            return
        if not self._sessions_manager.test_in_session(test, session):
            return
        if not self._sessions_manager.is_test_running(test, session):
            return
        self._tests_manager.complete_test(test, session)
        self._push_to_cache(token, result)
        self._update_test_state(result, session)

        session.last_completed_test = test
        session.recent_completed_count += 1
        self._sessions_manager.update_session(session)

        api = next((p for p in test.split("/") if p != ""), None)
        if session.recent_completed_count >= self._persisting_interval \
           or self._sessions_manager.is_api_complete(api, session):
            self.persist_session(session)

        if not self._sessions_manager.is_api_complete(api, session):
            return
        self.generate_report(token, api)

        test_state = session.test_state
        apis = list(test_state.keys())
        all_apis_complete = True
        for api in apis:
            if not self._sessions_manager.is_api_complete(api, session):
                all_apis_complete = False
        if not all_apis_complete:
            return
        self._sessions_manager.complete_session(token)
        self.create_info_file(session)

    def read_results(self, token, filter_path=None):
        filter_api = None
        if filter_path is not None:
            filter_api = next((p for p in filter_path.split("/")
                               if p is not None), None)
        results = self._read_from_cache(token)
        if results == []:
            results = self.load_results(token)
            self._set_session_cache(token, results)

        filtered_results = {}

        for api in results:
            if filter_api is not None and api.lower() != filter_api.lower():
                continue
            for result in results[api]:
                if filter_path is not None:
                    pattern = re.compile("^" + filter_path.replace(".", ""))
                    if pattern.match(result["test"].replace(".", "")) \
                       is None:
                        continue
                if api not in filtered_results:
                    filtered_results[api] = []
                filtered_results[api].append(result)

        return filtered_results

    def read_flattened_results(self, token):
        session = self._sessions_manager.read_session(token)
        return session.test_state

    def _update_test_state(self, result, session):
        api = next((p for p in result["test"].split("/") if p != ""), None)
        if "subtests" not in result:
            if result["status"] == "OK":
                session.test_state[api]["pass"] += 1
            elif result["status"] == "ERROR":
                session.test_state[api]["fail"] += 1
            elif result["status"] == "TIMEOUT":
                session.test_state[api]["timeout"] += 1
            elif result["status"] == "NOTRUN":
                session.test_state[api]["not_run"] += 1
        else:
            for test in result["subtests"]:
                if test["status"] == "PASS":
                    session.test_state[api]["pass"] += 1
                elif test["status"] == "FAIL":
                    session.test_state[api]["fail"] += 1
                elif test["status"] == "TIMEOUT":
                    session.test_state[api]["timeout"] += 1
                elif test["status"] == "NOTRUN":
                    session.test_state[api]["not_run"] += 1

        session.test_state[api]["complete"] += 1
        self._sessions_manager.update_session(session)

    def parse_test_state(self, results):
        test_state = {}
        for api in list(results.keys()):
            test_state[api] = {
                "pass": 0,
                "fail": 0,
                "timeout": 0,
                "not_run": 0,
                "total": len(results[api]),
                "complete": 0,
            }
            for result in results[api]:
                if "subtests" not in result:
                    if result["status"] == "OK":
                        test_state[api]["pass"] += 1
                    elif result["status"] == "ERROR":
                        test_state[api]["fail"] += 1
                    elif result["status"] == "TIMEOUT":
                        test_state[api]["timeout"] += 1
                    elif result["status"] == "NOTRUN":
                        test_state[api]["not_run"] += 1
                else:
                    for test in result["subtests"]:
                        if test["status"] == "PASS":
                            test_state[api]["pass"] += 1
                        elif test["status"] == "FAIL":
                            test_state[api]["fail"] += 1
                        elif test["status"] == "TIMEOUT":
                            test_state[api]["timeout"] += 1
                        elif test["status"] == "NOTRUN":
                            test_state[api]["not_run"] += 1
                test_state[api]["complete"] += 1
        return test_state

    def read_common_passed_tests(self, tokens=None):
        if tokens is None or len(tokens) == 0:
            return None

        session_results = []

        for token in tokens:
            session_result = self.read_results(token)
            session_results.append(session_result)

        passed_tests = {}
        failed_tests = {}

        for result in session_results:
            for api in result:
                if api not in passed_tests:
                    passed_tests[api] = []
                if api not in failed_tests:
                    failed_tests[api] = []

                for api_result in result[api]:
                    passed = True
                    for subtest in api_result["subtests"]:
                        if subtest["status"] == "PASS":
                            continue
                        passed = False
                        break

                    test = api_result["test"]

                    if passed:
                        if test in failed_tests[api]:
                            continue
                        if test in passed_tests[api]:
                            continue
                        passed_tests[api].append(test)
                    else:
                        if test in passed_tests[api]:
                            passed_tests[api].remove(test)
                        if test in failed_tests[api]:
                            continue
                        failed_tests[api].append(test)
        return passed_tests

    def read_results_wpt_report_uri(self, token, api):
        api_directory = os.path.join(self._results_directory_path, token, api)
        if not os.path.isdir(api_directory):
            return None
        return f"/results/{token}/{api}/all.html"

    def read_results_wpt_multi_report_uri(self, tokens, api):
        comparison_directory_name = self.get_comparison_identifier(tokens)

        relative_api_directory_path = os.path.join(comparison_directory_name,
                                                   api)

        api_directory_path = os.path.join(
            self._results_directory_path,
            relative_api_directory_path
        )

        if not os.path.isdir(api_directory_path):
            self.generate_multi_report(tokens, api)

        return f"/results/{relative_api_directory_path}/all.html"

    def delete_results(self, token):
        results_directory = os.path.join(self._results_directory_path, token)
        if not os.path.isdir(results_directory):
            return
        shutil.rmtree(results_directory)

    def persist_session(self, session):
        token = session.token
        if token not in self._results:
            return
        for api in list(self._results[token].keys())[:]:
            self.save_api_results(token, api)
        self.create_info_file(session)
        session.recent_completed_count = 0
        self._sessions_manager.update_session(session)

    def load_results(self, token):
        results_directory = os.path.join(self._results_directory_path, token)
        if not os.path.isdir(results_directory):
            return {}
        results = {}
        apis = os.listdir(results_directory)
        for api in apis:
            api_directory = os.path.join(results_directory, api)
            if not os.path.isdir(api_directory):
                continue
            files = os.listdir(api_directory)
            for file_name in files:
                if re.match(r"\w\w\d{1,3}\.json", file_name) is None:
                    continue
                file_path = os.path.join(api_directory, file_name)
                data = None
                with open(file_path) as file:
                    data = file.read()
                result = json.loads(data)
                results[api] = result["results"]
                break
        return results

    def _push_to_cache(self, token, result):
        if token is None:
            return
        if token not in self._results:
            self._results[token] = {}
        test = result["test"]
        api = next((p for p in test.split("/") if p != ""), None)
        if api not in self._results[token]:
            self._results[token][api] = []
        self._results[token][api].append(result)
        self._set_timeout(token)

    def _set_session_cache(self, token, results):
        if token is None:
            return
        self._results[token] = results
        self._set_timeout(token)

    def _read_from_cache(self, token):
        if token is None:
            return []
        if token not in self._results:
            return []
        self._set_timeout(token)
        return self._results[token]

    def _clear_session_cache(self, token):
        if token is None:
            return
        if token not in self._results:
            return
        del self._results[token]

    def _combine_results_by_api(self, result_a, result_b):
        combined_result = {}
        for api in result_a:
            if api in result_b:
                combined_result[api] = result_a[api] + result_b[api]
            else:
                combined_result[api] = result_a[api]

        for api in result_b:
            if api in combined_result:
                continue
            combined_result[api] = result_b[api]

        return combined_result

    def prepare_result(self, result):
        harness_status_map = {
            0: "OK",
            1: "ERROR",
            2: "TIMEOUT",
            3: "NOTRUN",
            "OK": "OK",
            "ERROR": "ERROR",
            "TIMEOUT": "TIMEOUT",
            "NOTRUN": "NOTRUN"
        }

        subtest_status_map = {
            0: "PASS",
            1: "FAIL",
            2: "TIMEOUT",
            3: "NOTRUN",
            "PASS": "PASS",
            "FAIL": "FAIL",
            "TIMEOUT": "TIMEOUT",
            "NOTRUN": "NOTRUN"
        }

        if "tests" in result:
            for test in result["tests"]:
                test["status"] = subtest_status_map[test["status"]]
                if "stack" in test:
                    del test["stack"]
            result["subtests"] = result["tests"]
            del result["tests"]

        if "stack" in result:
            del result["stack"]
        result["status"] = harness_status_map[result["status"]]

        return result

    def get_json_path(self, token, api):
        session = self._sessions_manager.read_session(token)
        api_directory = os.path.join(self._results_directory_path, token, api)

        browser = parse_user_agent(session.user_agent)
        abbreviation = abbreviate_browser_name(browser["name"])
        version = browser["version"]
        if "." in version:
            version = version.split(".")[0]
        version = version.zfill(2)
        file_name = abbreviation + version + ".json"

        return os.path.join(api_directory, file_name)

    def save_api_results(self, token, api):
        results = self._read_from_cache(token)
        if api not in results:
            return
        results = results[api]
        session = self._sessions_manager.read_session(token)
        self._ensure_results_directory_existence(api, token, session)

        file_path = self.get_json_path(token, api)
        file_exists = os.path.isfile(file_path)

        with open(file_path, "r+" if file_exists else "w") as file:
            api_results = None
            if file_exists:
                data = file.read()
                api_results = json.loads(data)
            else:
                api_results = {"results": []}

            api_results["results"] = api_results["results"] + results

            file.seek(0)
            file.truncate()
            file.write(json.dumps(api_results, indent=4, separators=(',', ': ')))

    def _ensure_results_directory_existence(self, api, token, session):
        directory = os.path.join(self._results_directory_path, token, api)
        if not os.path.exists(directory):
            os.makedirs(directory)

    def generate_report(self, token, api):
        file_path = self.get_json_path(token, api)
        dir_path = os.path.dirname(file_path)
        generate_report(
            input_json_directory_path=dir_path,
            output_html_directory_path=dir_path,
            spec_name=api
        )

    def generate_multi_report(self, tokens, api):
        comparison_directory_name = self.get_comparison_identifier(tokens)

        api_directory_path = os.path.join(
            self._results_directory_path,
            comparison_directory_name,
            api
        )

        if os.path.isdir(api_directory_path):
            return None

        os.makedirs(api_directory_path)

        result_json_files = []
        for token in tokens:
            result_json_files.append({
                "token": token,
                "path": self.get_json_path(token, api)
            })
        for file in result_json_files:
            if not os.path.isfile(file["path"]):
                return None
        generate_multi_report(
            output_html_directory_path=api_directory_path,
            spec_name=api,
            result_json_files=result_json_files
        )

    def get_comparison_identifier(self, tokens, ref_tokens=None):
        if ref_tokens is None:
            ref_tokens = []
        comparison_directory = "comparison"
        tokens.sort()
        for token in tokens:
            short_token = token.split("-")[0]
            comparison_directory += "-" + short_token
        hash = hashlib.sha1()
        ref_tokens.sort()
        for token in ref_tokens:
            hash.update(token.encode("utf-8"))
        for token in tokens:
            hash.update(token.encode("utf-8"))
        hash = hash.hexdigest()
        comparison_directory += hash[0:8]
        return comparison_directory

    def create_info_file(self, session):
        token = session.token
        info_file_path = os.path.join(
            self._results_directory_path,
            token,
            "info.json"
        )
        info = serialize_session(session)
        del info["running_tests"]
        del info["pending_tests"]

        file_content = json.dumps(info, indent=2)
        with open(info_file_path, "w+") as file:
            file.write(file_content)

    def export_results_api_json(self, token, api):
        results = self.read_results(token)
        if api in results:
            return json.dumps({"results": results[api]}, indent=4)

        file_path = self.get_json_path(token, api)
        if not os.path.isfile(file_path):
            return None

        with open(file_path) as file:
            blob = file.read()
            return blob

    def export_results_all_api_jsons(self, token):
        self._sessions_manager.read_session(token)
        results_directory = os.path.join(self._results_directory_path, token)
        results = self.read_results(token)

        zip_file_name = str(time.time()) + ".zip"
        zip = zipfile.ZipFile(zip_file_name, "w")
        for api, result in results.items():
            zip.writestr(
                api + ".json",
                json.dumps({"results": result}, indent=4),
                zipfile.ZIP_DEFLATED
            )

        results_directory = os.path.join(self._results_directory_path, token)
        if os.path.isdir(results_directory):
            persisted_apis = os.listdir(results_directory)

            for api in persisted_apis:
                if api in results:
                    continue
                blob = self.export_results_api_json(token, api)
                if blob is None:
                    continue
                zip.writestr(api + ".json", blob, zipfile.ZIP_DEFLATED)

        zip.close()

        with open(zip_file_name, "rb") as file:
            blob = file.read()
            os.remove(zip_file_name)

            return blob

    def export_results(self, token):
        if token is None:
            return
        session = self._sessions_manager.read_session(token)
        if session.status != COMPLETED:
            return None

        session_results_directory = os.path.join(self._results_directory_path,
                                                 token)
        if not os.path.isdir(session_results_directory):
            return None

        zip_file_name = str(time.time()) + ".zip"
        zip = zipfile.ZipFile(zip_file_name, "w")
        for root, dirs, files in os.walk(session_results_directory):
            for file in files:
                file_name = os.path.join(root.split(token)[1], file)
                file_path = os.path.join(root, file)
                zip.write(file_path, file_name, zipfile.ZIP_DEFLATED)
        zip.close()

        with open(zip_file_name) as file:
            blob = file.read()
            os.remove(zip_file_name)

            return blob

    def export_results_overview(self, token):
        session = self._sessions_manager.read_session(token)
        if session is None:
            raise NotFoundException(f"Could not find session {token}")

        tmp_file_name = str(time.time()) + ".zip"
        zip = zipfile.ZipFile(tmp_file_name, "w")

        flattened_results = self.read_flattened_results(token)
        results_script = "const results = " + json.dumps(flattened_results,
                                                         indent=4)
        zip.writestr("results.json.js", results_script)

        session_dict = serialize_session(session)
        del session_dict["running_tests"]
        del session_dict["pending_tests"]
        details_script = "const details = " + json.dumps(session_dict,
                                                         indent=4)
        zip.writestr("details.json.js", details_script)

        for root, dirs, files in os.walk(os.path.join(WAVE_SRC_DIR, "export")):
            for file in files:
                file_name = os.path.join(root.split("export")[1], file)
                file_path = os.path.join(root, file)
                zip.write(file_path, file_name, zipfile.ZIP_DEFLATED)

        zip.close()

        with open(tmp_file_name, "rb") as file:
            blob = file.read()

            self.remove_tmp_files()

            return blob

    def is_import_results_enabled(self):
        return self._import_results_enabled

    def are_reports_enabled(self):
        return self._reports_enabled

    def load_session_from_info_file(self, info_file_path):
        if not os.path.isfile(info_file_path):
            return None

        with open(info_file_path) as info_file:
            data = info_file.read()
            info_file.close()
            info = json.loads(str(data))
            return deserialize_session(info)

    def import_results(self, blob):
        if not self.is_import_results_enabled:
            raise PermissionDeniedException()
        tmp_file_name = f"{str(time.time())}.zip"

        with open(tmp_file_name, "w") as file:
            file.write(blob)

        zip = zipfile.ZipFile(tmp_file_name)
        if "info.json" not in zip.namelist():
            raise InvalidDataException("Invalid session ZIP!")
        zipped_info = zip.open("info.json")
        info = zipped_info.read()
        zipped_info.close()
        parsed_info = json.loads(info)
        token = parsed_info["token"]
        session = self._sessions_manager.read_session(token)
        if session is not None:
            raise DuplicateException("Session already exists!")
        destination_path = os.path.join(self._results_directory_path, token)
        os.makedirs(destination_path)
        zip.extractall(destination_path)
        self.remove_tmp_files()
        self.load_results(token)
        return token

    def import_results_api_json(self, token, api, blob):
        if not self.is_import_results_enabled:
            raise PermissionDeniedException()
        destination_path = os.path.join(self._results_directory_path, token, api)
        files = os.listdir(destination_path)
        file_name = ""
        for file in files:
            if RESULTS_FILE_PATTERN.match(file):
                file_name = file
                break
        destination_file_path = os.path.join(destination_path, file_name)
        with open(destination_file_path, "wb") as file:
            file.write(blob)

        self.generate_report(token, api)

        session = self._sessions_manager.read_session(token)
        if session is None:
            raise NotFoundException()

        results = self.load_results(token)
        test_state = self.parse_test_state(results)
        session.test_state = test_state

        self._sessions_manager.update_session(session)

    def remove_tmp_files(self):
        files = os.listdir(".")

        for file in files:
            if re.match(r"\d{10}\.\d{2}\.zip", file) is None:
                continue
            os.remove(file)

    def _set_timeout(self, token):
        if token in self._timeouts:
            self._timeouts[token].cancel()

        def handler(self, token):
            self._clear_session_cache(token)

        self._timeouts[token] = Timer(SESSION_RESULTS_TIMEOUT, handler, [self, token])
