# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from xml.etree import ElementTree

from . import base


def format_test_id(test_id):
    """Take a test id and return something that looks a bit like
    a class path"""
    if not isinstance(test_id, str):
        # Not sure how to deal with reftests yet
        raise NotImplementedError

    # Turn a path into something like a class heirachy
    return test_id.replace(".", "_").replace("/", ".")


class XUnitFormatter(base.BaseFormatter):
    """Formatter that produces XUnit-style XML output.

    The tree is created in-memory so this formatter may be problematic
    with very large log files.

    Note that the data model isn't a perfect match. In
    particular XUnit assumes that each test has a unittest-style
    class name and function name, which isn't the case for us. The
    implementation currently replaces path names with something that
    looks like class names, but this doesn't work for test types that
    actually produce class names, or for test types that have multiple
    components in their test id (e.g. reftests)."""

    def __init__(self):
        self.tree = ElementTree.ElementTree()
        self.root = None
        self.suite_start_time = None
        self.test_start_time = None

        self.tests_run = 0
        self.errors = 0
        self.failures = 0
        self.skips = 0

    def suite_start(self, data):
        self.root = ElementTree.Element("testsuite")
        self.tree.root = self.root
        self.suite_start_time = data["time"]

    def test_start(self, data):
        self.tests_run += 1
        self.test_start_time = data["time"]

    def _create_result(self, data):
        test = ElementTree.SubElement(self.root, "testcase")
        name = format_test_id(data["test"])
        extra = data.get("extra") or {}
        test.attrib["classname"] = extra.get("class_name") or name

        if "subtest" in data:
            test.attrib["name"] = data["subtest"]
            # We generally don't know how long subtests take
            test.attrib["time"] = "0"
        else:
            if "." in name:
                test_name = name.rsplit(".", 1)[1]
            else:
                test_name = name
            test.attrib["name"] = extra.get("method_name") or test_name
            test.attrib["time"] = "%.2f" % (
                (data["time"] - self.test_start_time) / 1000.0
            )

        if "expected" in data and data["expected"] != data["status"]:
            if data["status"] in ("NOTRUN", "ASSERT", "ERROR"):
                result = ElementTree.SubElement(test, "error")
                self.errors += 1
            else:
                result = ElementTree.SubElement(test, "failure")
                self.failures += 1

            result.attrib["message"] = "Expected %s, got %s" % (
                data["expected"],
                data["status"],
            )
            result.text = "%s\n%s" % (data.get("stack", ""), data.get("message", ""))

        elif data["status"] == "SKIP":
            result = ElementTree.SubElement(test, "skipped")
            self.skips += 1

    def test_status(self, data):
        self._create_result(data)

    def test_end(self, data):
        self._create_result(data)

    def suite_end(self, data):
        self.root.attrib.update(
            {
                "tests": str(self.tests_run),
                "errors": str(self.errors),
                "failures": str(self.failures),
                "skips": str(self.skips),
                "time": "%.2f" % ((data["time"] - self.suite_start_time) / 1000.0),
            }
        )
        xml_string = ElementTree.tostring(self.root, encoding="utf8")
        # pretty printing can not be done from xml.etree
        from xml.dom import minidom

        return minidom.parseString(xml_string).toprettyxml()
