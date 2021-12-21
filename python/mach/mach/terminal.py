# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

"""This file contains code for interacting with terminals.

All the terminal interaction code is consolidated so the complexity can be in
one place, away from code that is commonly looked at.
"""

from __future__ import absolute_import, print_function, unicode_literals

import logging
import sys

from six.moves import range


class LoggingHandler(logging.Handler):
    """Custom logging handler that works with terminal window dressing.

    This is alternative terminal logging handler which contains smarts for
    emitting terminal control characters properly. Currently, it has generic
    support for "footer" elements at the bottom of the screen. Functionality
    can be added when needed.
    """
    def __init__(self):
        logging.Handler.__init__(self)

        self.fh = sys.stdout
        self.footer = None

    def flush(self):
        self.acquire()

        try:
            self.fh.flush()
        finally:
            self.release()

    def emit(self, record):
        msg = self.format(record)

        if self.footer:
            self.footer.clear()

        self.fh.write(msg)
        self.fh.write('\n')

        if self.footer:
            self.footer.draw()

        # If we don't flush, the footer may not get drawn.
        self.flush()


class TerminalFooter(object):
    """Represents something drawn on the bottom of a terminal."""
    def __init__(self, terminal):
        self.t = terminal
        self.fh = sys.stdout

    def _clear_lines(self, n):
        for i in range(n):
            self.fh.write(self.t.move_x(0))
            self.fh.write(self.t.clear_eol())
            self.fh.write(self.t.move_up())

        self.fh.write(self.t.move_down())
        self.fh.write(self.t.move_x(0))

    def clear(self):
        raise Exception('clear() must be implemented.')

    def draw(self):
        raise Exception('draw() must be implemented.')
