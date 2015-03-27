# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from collections import (
    defaultdict,
    namedtuple,
)

from mozlog.structured.structuredlog import log_levels

RunSummary = namedtuple("RunSummary",
                        ("unexpected_statuses",
                         "expected_statuses",
                         "log_level_counts",
                         "action_counts"))

class StatusHandler(object):
    """A handler used to determine an overall status for a test run according
    to a sequence of log messages."""

    def __init__(self):
        # The count of each type of unexpected result status (includes tests and subtests)
        self.unexpected_statuses = defaultdict(int)
        # The count of each type of expected result status (includes tests and subtests)
        self.expected_statuses = defaultdict(int)
        # The count of actions logged
        self.action_counts = defaultdict(int)
        # The count of messages logged at each log level
        self.log_level_counts = defaultdict(int)


    def __call__(self, data):
        action = data['action']
        self.action_counts[action] += 1

        if action == 'log':
            self.log_level_counts[data['level']] += 1

        if action in ('test_status', 'test_end'):
            status = data['status']
            if 'expected' in data:
                self.unexpected_statuses[status] += 1
            else:
                self.expected_statuses[status] += 1


    def summarize(self):
        return RunSummary(
            dict(self.unexpected_statuses),
            dict(self.expected_statuses),
            dict(self.log_level_counts),
            dict(self.action_counts),
        )
