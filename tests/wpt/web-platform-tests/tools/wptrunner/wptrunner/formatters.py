import json

from mozlog.structured.formatters.base import BaseFormatter


class WptreportFormatter(BaseFormatter):
    """Formatter that produces results in the format that wpreport expects."""

    def __init__(self):
        self.raw_results = {}

    def suite_end(self, data):
        results = {}
        results["results"] = []
        for test_name in self.raw_results:
            result = {"test": test_name}
            result.update(self.raw_results[test_name])
            results["results"].append(result)
        return json.dumps(results)

    def find_or_create_test(self, data):
        test_name = data["test"]
        if test_name not in self.raw_results:
            self.raw_results[test_name] = {
                "subtests": [],
                "status": "",
                "message": None
            }
        return self.raw_results[test_name]

    def create_subtest(self, data):
        test = self.find_or_create_test(data)
        subtest_name = data["subtest"]

        subtest = {
            "name": subtest_name,
            "status": "",
            "message": None
        }
        test["subtests"].append(subtest)

        return subtest

    def test_status(self, data):
        subtest = self.create_subtest(data)
        subtest["status"] = data["status"]
        if "message" in data:
            subtest["message"] = data["message"]

    def test_end(self, data):
        test = self.find_or_create_test(data)
        test["status"] = data["status"]
        if "message" in data:
            test["message"] = data["message"]
