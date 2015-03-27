# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
import json

def read(log_f, raise_on_error=False):
    """Return a generator that will return the entries in a structured log file.
    Note that the caller must not close the file whilst the generator is still
    in use.

    :param log_f: file-like object containing the raw log entries, one per line
    :param raise_on_error: boolean indicating whether ValueError should be raised
                           for lines that cannot be decoded."""
    while True:
        line = log_f.readline()
        if not line:
            # This allows log_f to be a stream like stdout
            break
        try:
            yield json.loads(line)
        except ValueError:
            if raise_on_error:
                raise


def imap_log(log_iter, action_map):
    """Create an iterator that will invoke a callback per action for each item in a
    iterable containing structured log entries

    :param log_iter: Iterator returning structured log entries
    :param action_map: Dictionary mapping action name to callback function. Log items
                       with actions not in this dictionary will be skipped.
    """
    for item in log_iter:
        if item["action"] in action_map:
            yield action_map[item["action"]](item)

def each_log(log_iter, action_map):
    """Call a callback for each item in an iterable containing structured
    log entries

    :param log_iter: Iterator returning structured log entries
    :param action_map: Dictionary mapping action name to callback function. Log items
                       with actions not in this dictionary will be skipped.
    """
    for item in log_iter:
        if item["action"] in action_map:
            action_map[item["action"]](item)

class LogHandler(object):
    """Base class for objects that act as log handlers. A handler is a callable
    that takes a log entry as the only argument.

    Subclasses are expected to provide a method for each action type they
    wish to handle, each taking a single argument for the test data.
    For example a trivial subclass that just produces the id of each test as
    it starts might be::

      class StartIdHandler(LogHandler):
          def test_start(data):
              #For simplicity in the example pretend the id is always a string
              return data["test"]
    """

    def __call__(self, data):
        if hasattr(self, data["action"]):
            handler = getattr(self, data["action"])
            return handler(data)

def handle_log(log_iter, handler):
    """Call a handler for each item in a log, discarding the return value"""
    for item in log_iter:
        handler(item)
