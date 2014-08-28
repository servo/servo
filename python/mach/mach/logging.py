# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

# This file contains logging functionality for mach. It essentially provides
# support for a structured logging framework built on top of Python's built-in
# logging framework.

from __future__ import absolute_import, unicode_literals

try:
    import blessings
except ImportError:
    blessings = None

import json
import logging
import sys
import time


def format_seconds(total):
    """Format number of seconds to MM:SS.DD form."""

    minutes, seconds = divmod(total, 60)

    return '%2d:%05.2f' % (minutes, seconds)


class ConvertToStructuredFilter(logging.Filter):
    """Filter that converts unstructured records into structured ones."""
    def filter(self, record):
        if hasattr(record, 'action') and hasattr(record, 'params'):
            return True

        record.action = 'unstructured'
        record.params = {'msg': record.getMessage()}
        record.msg = '{msg}'

        return True


class StructuredJSONFormatter(logging.Formatter):
    """Log formatter that writes a structured JSON entry."""

    def format(self, record):
        action = getattr(record, 'action', 'UNKNOWN')
        params = getattr(record, 'params', {})

        return json.dumps([record.created, action, params])


class StructuredHumanFormatter(logging.Formatter):
    """Log formatter that writes structured messages for humans.

    It is important that this formatter never be added to a logger that
    produces unstructured/classic log messages. If it is, the call to format()
    could fail because the string could contain things (like JSON) that look
    like formatting character sequences.

    Because of this limitation, format() will fail with a KeyError if an
    unstructured record is passed or if the structured message is malformed.
    """
    def __init__(self, start_time, write_interval=False, write_times=True):
        self.start_time = start_time
        self.write_interval = write_interval
        self.write_times = write_times
        self.last_time = None

    def format(self, record):
        f = record.msg.format(**record.params)

        if not self.write_times:
            return f

        elapsed = self._time(record)

        return '%s %s' % (format_seconds(elapsed), f)

    def _time(self, record):
        t = record.created - self.start_time

        if self.write_interval and self.last_time is not None:
            t = record.created - self.last_time

        self.last_time = record.created

        return t


class StructuredTerminalFormatter(StructuredHumanFormatter):
    """Log formatter for structured messages writing to a terminal."""

    def set_terminal(self, terminal):
        self.terminal = terminal

    def format(self, record):
        f = record.msg.format(**record.params)

        if not self.write_times:
            return f

        t = self.terminal.blue(format_seconds(self._time(record)))

        return '%s %s' % (t, self._colorize(f))

    def _colorize(self, s):
        if not self.terminal:
            return s

        result = s

        reftest = s.startswith('REFTEST ')
        if reftest:
            s = s[8:]

        if s.startswith('TEST-PASS'):
            result = self.terminal.green(s[0:9]) + s[9:]
        elif s.startswith('TEST-UNEXPECTED'):
            result = self.terminal.red(s[0:20]) + s[20:]
        elif s.startswith('TEST-START'):
            result = self.terminal.yellow(s[0:10]) + s[10:]
        elif s.startswith('TEST-INFO'):
            result = self.terminal.yellow(s[0:9]) + s[9:]

        if reftest:
            result = 'REFTEST ' + result

        return result


class LoggingManager(object):
    """Holds and controls global logging state.

    An application should instantiate one of these and configure it as needed.

    This class provides a mechanism to configure the output of logging data
    both from mach and from the overall logging system (e.g. from other
    modules).
    """

    def __init__(self):
        self.start_time = time.time()

        self.json_handlers = []
        self.terminal_handler = None
        self.terminal_formatter = None

        self.root_logger = logging.getLogger()
        self.root_logger.setLevel(logging.DEBUG)

        # Installing NullHandler on the root logger ensures that *all* log
        # messages have at least one handler. This prevents Python from
        # complaining about "no handlers could be found for logger XXX."
        self.root_logger.addHandler(logging.NullHandler())

        self.mach_logger = logging.getLogger('mach')
        self.mach_logger.setLevel(logging.DEBUG)

        self.structured_filter = ConvertToStructuredFilter()

        self.structured_loggers = [self.mach_logger]

        self._terminal = None

    @property
    def terminal(self):
        if not self._terminal and blessings:
            # Sometimes blessings fails to set up the terminal. In that case,
            # silently fail.
            try:
                terminal = blessings.Terminal(stream=sys.stdout)

                if terminal.is_a_tty:
                    self._terminal = terminal
            except Exception:
                pass

        return self._terminal

    def add_json_handler(self, fh):
        """Enable JSON logging on the specified file object."""

        # Configure the consumer of structured messages.
        handler = logging.StreamHandler(stream=fh)
        handler.setFormatter(StructuredJSONFormatter())
        handler.setLevel(logging.DEBUG)

        # And hook it up.
        for logger in self.structured_loggers:
            logger.addHandler(handler)

        self.json_handlers.append(handler)

    def add_terminal_logging(self, fh=sys.stdout, level=logging.INFO,
            write_interval=False, write_times=True):
        """Enable logging to the terminal."""

        formatter = StructuredHumanFormatter(self.start_time,
            write_interval=write_interval, write_times=write_times)

        if self.terminal:
            formatter = StructuredTerminalFormatter(self.start_time,
                write_interval=write_interval, write_times=write_times)
            formatter.set_terminal(self.terminal)

        handler = logging.StreamHandler(stream=fh)
        handler.setFormatter(formatter)
        handler.setLevel(level)

        for logger in self.structured_loggers:
            logger.addHandler(handler)

        self.terminal_handler = handler
        self.terminal_formatter = formatter

    def replace_terminal_handler(self, handler):
        """Replace the installed terminal handler.

        Returns the old handler or None if none was configured.
        If the new handler is None, removes any existing handler and disables
        logging to the terminal.
        """
        old = self.terminal_handler

        if old:
            for logger in self.structured_loggers:
                logger.removeHandler(old)

        if handler:
            for logger in self.structured_loggers:
                logger.addHandler(handler)

        self.terminal_handler = handler

        return old

    def enable_unstructured(self):
        """Enable logging of unstructured messages."""
        if self.terminal_handler:
            self.terminal_handler.addFilter(self.structured_filter)
            self.root_logger.addHandler(self.terminal_handler)

    def disable_unstructured(self):
        """Disable logging of unstructured messages."""
        if self.terminal_handler:
            self.terminal_handler.removeFilter(self.structured_filter)
            self.root_logger.removeHandler(self.terminal_handler)

    def register_structured_logger(self, logger):
        """Register a structured logger.

        This needs to be called for all structured loggers that don't chain up
        to the mach logger in order for their output to be captured.
        """
        self.structured_loggers.append(logger)
