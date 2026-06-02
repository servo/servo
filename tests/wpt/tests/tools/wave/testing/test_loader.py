# mypy: allow-untyped-defs

import os
import re

AUTOMATIC = "automatic"
MANUAL = "manual"

TEST_TYPES = [AUTOMATIC, MANUAL]


class TestLoader:
    def initialize(
        self,
        exclude_list_file_path,
        include_list_file_path,
        results_manager,
        api_titles
    ):
        self._exclude_list_file_path = exclude_list_file_path
        self._include_list_file_path = include_list_file_path
        self._results_manager = results_manager
        self._tests = {}
        self._tests[AUTOMATIC] = {}
        self._tests[MANUAL] = {}
        self._api_titles = api_titles

    def load_tests(self, tests):
        include_list = self._load_test_list(self._include_list_file_path)
        exclude_list = self._load_test_list(self._exclude_list_file_path)

        if "testharness" in tests:
            self._tests[AUTOMATIC] = self._load_tests(
                tests=tests["testharness"],
                exclude_list=exclude_list
            )

        if "manual" in tests:
            self._tests[MANUAL] = self._load_tests(
                tests=tests["manual"],
                include_list=include_list
            )

        for api in self._tests[AUTOMATIC]:
            for test_path in self._tests[AUTOMATIC][api][:]:
                if "manual" not in test_path:
                    continue
                self._tests[AUTOMATIC][api].remove(test_path)

                if not self._is_valid_test(test_path,
                                           include_list=include_list):
                    continue

                if api not in self._tests[MANUAL]:
                    self._tests[MANUAL][api] = []
                self._tests[MANUAL][api].append(test_path)

    def _load_tests(self, tests, exclude_list=None, include_list=None):
        loaded_tests = {}

        def get_next_part(tests):
            paths = []
            for test in tests:
                if isinstance(tests[test], dict):
                    subs = get_next_part(tests[test])
                    for sub in subs:
                        if sub is None:
                            continue
                        paths.append(test + "/" + sub)
                    continue
                if test.endswith(".html"):
                    paths.append(test)
                    continue
                if test.endswith(".js"):
                    for element in tests[test][1:]:
                        paths.append(element[0])
                    continue
            return paths

        test_paths = get_next_part(tests)
        for test_path in test_paths:
            if not test_path.startswith("/"):
                test_path = "/" + test_path
            if self._is_valid_test(test_path, exclude_list, include_list):
                api_name = self._parse_api_name(test_path)
                if api_name not in loaded_tests:
                    loaded_tests[api_name] = []
                loaded_tests[api_name].append(test_path)
        return loaded_tests

    def _parse_api_name(self, test_path):
        for part in test_path.split("/"):
            if part == "":
                continue
            return part

    def _is_valid_test(self, test_path, exclude_list=None, include_list=None):
        is_valid = True

        if include_list is not None and len(include_list) > 0:
            is_valid = False
            for include_test in include_list:
                include_test = include_test.split("?")[0]
                pattern = re.compile("^" + include_test)
                if pattern.match(test_path) is not None:
                    is_valid = True
                    break

        if not is_valid:
            return is_valid

        if exclude_list is not None and len(exclude_list) > 0:
            is_valid = True
            for exclude_test in exclude_list:
                exclude_test = exclude_test.split("?")[0]
                pattern = re.compile("^" + exclude_test)
                if pattern.match(test_path) is not None:
                    is_valid = False
                    break

        return is_valid

    def _load_test_list(self, file_path):
        tests = []
        if not os.path.isfile(file_path):
            return tests

        file_content = None
        with open(file_path) as file_handle:
            file_content = file_handle.read()

        for line in file_content.split():
            line = line.replace(" ", "")
            line = re.sub(r"^#", "", line)
            if line == "":
                continue
            tests.append(line)

        return tests

    def get_tests(
        self,
        test_types=None,
        include_list=None,
        exclude_list=None,
        reference_tokens=None
    ):
        if test_types is None:
            test_types = [AUTOMATIC, MANUAL]
        if include_list is None:
            include_list = []
        if exclude_list is None:
            exclude_list = []
        if reference_tokens is None:
            reference_tokens = []

        loaded_tests = {}

        reference_results = self._results_manager.read_common_passed_tests(
            reference_tokens)

        for test_type in test_types:
            if test_type not in TEST_TYPES:
                continue
            for api in self._tests[test_type]:
                for test_path in self._tests[test_type][api]:
                    if not self._is_valid_test(test_path, exclude_list,
                                               include_list):
                        continue
                    if reference_results is not None and \
                       (api not in reference_results or
                       (api in reference_results and test_path not in reference_results[api])):
                        continue
                    if api not in loaded_tests:
                        loaded_tests[api] = []
                    loaded_tests[api].append(test_path)
        return loaded_tests

    def get_apis(self):
        apis = []
        for test_type in TEST_TYPES:
            for api in self._tests[test_type]:
                in_list = False
                for item in apis:
                    if item["path"] == "/" + api:
                        in_list = True
                        break
                if in_list:
                    continue
                title = None
                for item in self._api_titles:
                    if item["path"] == "/" + api:
                        title = item["title"]
                        break

                if title is None:
                    apis.append({"title": api, "path": "/" + api})
                else:
                    apis.append({"title": title, "path": "/" + api})
        return apis
