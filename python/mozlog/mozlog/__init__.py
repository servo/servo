# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

"""
Mozlog aims to standardize log formatting within Mozilla.

It simply wraps Python's logging_ module and adds a few convenience methods
for logging test results and events.

The structured submodule takes a different approach and implements a
JSON-based logging protocol designed for recording test results."""

from logger import *
from loglistener import LogMessageServer
from loggingmixin import LoggingMixin

try:
    import structured
except ImportError:
    # Structured logging doesn't work on python 2.6 which is still used on some
    # legacy test machines; https://bugzilla.mozilla.org/show_bug.cgi?id=864866
    # Once we move away from Python 2.6, please cleanup devicemanager.py's
    # exception block
    pass

