# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from mozlog.formatters import base
import mozlog.commandline
import collections

class FriendlyFormatter(base.BaseFormatter):
    """Formatter designed to produce output in a friendly and readable format."""
    def __init__(self):
        self.number_of_tests = 0
        self.completed_tests = 0
        self.need_to_erase_last_line = False
        self.current_display = ""
        self.running_tests = {}
        self.subtest_failures = collections.defaultdict(list)
        self.tests_with_failing_subtests = []

        self.expected  = {
            'OK': 0,
            'PASS': 0,
            'FAIL': 0,
            'ERROR': 0,
            'TIMEOUT': 0,
            'SKIP': 0,
            'CRASH': 0,
        }

        self.unexpected_tests  = {
            'OK': [],
            'PASS': [],
            'FAIL': [],
            'ERROR': [],
            'TIMEOUT': [],
            'CRASH': [],
        }

    def text_to_erase_display(self):
        if not self.current_display:
            return ""
        return ("\033[F" + "\033[K") * len(self.current_display.splitlines())

    def generate_output(self, text=None, new_display=None):
        output = self.text_to_erase_display()
        if text:
            output += text
        if new_display != None:
            self.current_display = new_display
        return output + self.current_display

    def build_status_line(self):
        new_display = "\n  [%i/%i] " % (self.completed_tests, self.number_of_tests)
        if self.running_tests:
            indent = " " * (len(new_display) - 1)
            return new_display + \
                ("\n%s" % indent).join(self.running_tests.values()) + "\n"
        else:
            return new_display + "No tests running.\n"

    def suite_start(self, data):
        self.number_of_tests = len(data["tests"])
        self.start_time = data["time"]
        return "Running %i tests in %s\n" % (self.number_of_tests, data[u'source'])

    def test_start(self, data):
        self.running_tests[data['thread']] = data['test']
        return self.generate_output(text=None,
                                    new_display=self.build_status_line())

    def test_end(self, data):
        self.completed_tests += 1
        test_status = data["status"]
        test_name = data["test"]
        had_unexpected_test_result = "expected" in data
        subtest_failures = self.subtest_failures.pop(test_name, [])

        del self.running_tests[data['thread']]
        new_display = self.build_status_line()

        if not had_unexpected_test_result and not subtest_failures:
            self.expected[test_status] += 1
            return self.generate_output(text=None, new_display=new_display)

        if had_unexpected_test_result:
            self.unexpected_tests[test_status].append(data)
        if subtest_failures:
            self.tests_with_failing_subtests.append(test_name)

        output = u"  %(status)s %(test)s\n" % data
        subtest_template = u"    \u2022 %s %s\n" \
                           u"      %s\n"
        for failure in subtest_failures:
            output += subtest_template % \
                (failure['status'],
                 failure['subtest'].encode('utf-8').encode('string-escape'),
                 failure['message'])
            if 'stack' in failure:
                output += u"        @ %s\n" % failure['stack'].splitlines()[0]

        return self.generate_output(text=output, new_display=new_display)

    def test_status(self, data):
        if "expected" in data:
            self.subtest_failures[data["test"]].append(data)

    def suite_end(self, data):
        self.end_time = data["time"]

        output = u"\nRan %i tests finished in %.1f seconds.\n" % \
            (self.completed_tests, (self.end_time - self.start_time) / 1000)
        output += u"  \u2022 %i ran as expected. %i tests skipped.\n" % \
            (sum(self.expected.values()), self.expected['SKIP'])

        def text_for_unexpected_list(text, section):
            tests = self.unexpected_tests[section]
            if not tests:
                return u""
            return u"  \u2022 %i tests %s:\n      " % (len(tests), text) + \
                u"\n      ".join("%(test)s [expected %(expected)s]" % data for data in tests) + "\n"

        output += text_for_unexpected_list(u"had crashed unexpectedly", 'CRASH')
        output += text_for_unexpected_list(u"had errors unexpectedly", 'ERROR')
        output += text_for_unexpected_list(u"failed unexpectedly", 'FAIL')
        output += text_for_unexpected_list(u"timed out unexpectedly", 'TIMEOUT')
        output += text_for_unexpected_list(u"passed unexpectedly", 'PASS')
        output += text_for_unexpected_list(u"unexpectedly okay", 'OK')

        if self.tests_with_failing_subtests:
            output += u"  \u2022 %i tests had unexpected subtest results\n" \
                % len(self.tests_with_failing_subtests) + u"      " + \
                u"\n      ".join("%s" % data for data \
                                     in self.tests_with_failing_subtests) + "\n"

        return self.generate_output(text=output, new_display="")
