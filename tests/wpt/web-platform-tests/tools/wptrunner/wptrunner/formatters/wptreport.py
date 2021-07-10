import json
import re
import sys

from mozlog.structured.formatters.base import BaseFormatter
from ..executors.base import strip_server


LONE_SURROGATE_RE = re.compile(u"[\uD800-\uDFFF]")


def surrogate_replacement_ucs4(match):
    return "U+" + hex(ord(match.group()))[2:]


class SurrogateReplacementUcs2(object):
    def __init__(self):
        self.skip = False

    def __call__(self, match):
        char = match.group()

        if self.skip:
            self.skip = False
            return char

        is_low = 0xD800 <= ord(char) <= 0xDBFF

        escape = True
        if is_low:
            next_idx = match.end()
            if next_idx < len(match.string):
                next_char = match.string[next_idx]
                if 0xDC00 <= ord(next_char) <= 0xDFFF:
                    escape = False

        if not escape:
            self.skip = True
            return char

        return "U+" + hex(ord(match.group()))[2:]


if sys.maxunicode == 0x10FFFF:
    surrogate_replacement = surrogate_replacement_ucs4
else:
    surrogate_replacement = SurrogateReplacementUcs2()


def replace_lone_surrogate(data):
    return LONE_SURROGATE_RE.subn(surrogate_replacement, data)[0]


class WptreportFormatter(BaseFormatter):
    """Formatter that produces results in the format that wptreport expects."""

    def __init__(self):
        self.raw_results = {}
        self.results = {}

    def suite_start(self, data):
        if 'run_info' in data:
            self.results['run_info'] = data['run_info']
        self.results['time_start'] = data['time']
        self.results["results"] = []

    def suite_end(self, data):
        self.results['time_end'] = data['time']
        for test_name in self.raw_results:
            result = {"test": test_name}
            result.update(self.raw_results[test_name])
            self.results["results"].append(result)
        return json.dumps(self.results) + "\n"

    def find_or_create_test(self, data):
        test_name = data["test"]
        if test_name not in self.raw_results:
            self.raw_results[test_name] = {
                "subtests": [],
                "status": "",
                "message": None
            }
        return self.raw_results[test_name]

    def test_start(self, data):
        test = self.find_or_create_test(data)
        test["start_time"] = data["time"]

    def create_subtest(self, data):
        test = self.find_or_create_test(data)
        subtest_name = replace_lone_surrogate(data["subtest"])

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
        if "expected" in data:
            subtest["expected"] = data["expected"]
        if "known_intermittent" in data:
            subtest["known_intermittent"] = data["known_intermittent"]
        if "message" in data:
            subtest["message"] = replace_lone_surrogate(data["message"])

    def test_end(self, data):
        test = self.find_or_create_test(data)
        start_time = test.pop("start_time")
        test["duration"] = data["time"] - start_time
        test["status"] = data["status"]
        if "expected" in data:
            test["expected"] = data["expected"]
        if "known_intermittent" in data:
            test["known_intermittent"] = data["known_intermittent"]
        if "message" in data:
            test["message"] = replace_lone_surrogate(data["message"])
        if "reftest_screenshots" in data.get("extra", {}):
            test["screenshots"] = {
                strip_server(item["url"]): "sha1:" + item["hash"]
                for item in data["extra"]["reftest_screenshots"]
                if type(item) == dict
            }
        test_name = data["test"]
        result = {"test": data["test"]}
        result.update(self.raw_results[test_name])
        self.results["results"].append(result)
        self.raw_results.pop(test_name)

    def assertion_count(self, data):
        test = self.find_or_create_test(data)
        test["asserts"] = {
            "count": data["count"],
            "min": data["min_expected"],
            "max": data["max_expected"]
        }

    def lsan_leak(self, data):
        if "lsan_leaks" not in self.results:
            self.results["lsan_leaks"] = []
        lsan_leaks = self.results["lsan_leaks"]
        lsan_leaks.append({"frames": data["frames"],
                           "scope": data["scope"],
                           "allowed_match": data.get("allowed_match")})

    def find_or_create_mozleak(self, data):
        if "mozleak" not in self.results:
            self.results["mozleak"] = {}
        scope = data["scope"]
        if scope not in self.results["mozleak"]:
            self.results["mozleak"][scope] = {"objects": [], "total": []}
        return self.results["mozleak"][scope]

    def mozleak_object(self, data):
        scope_data = self.find_or_create_mozleak(data)
        scope_data["objects"].append({"process": data["process"],
                                      "name": data["name"],
                                      "allowed": data.get("allowed", False),
                                      "bytes": data["bytes"]})

    def mozleak_total(self, data):
        scope_data = self.find_or_create_mozleak(data)
        scope_data["total"].append({"bytes": data["bytes"],
                                    "threshold": data.get("threshold", 0),
                                    "process": data["process"]})
