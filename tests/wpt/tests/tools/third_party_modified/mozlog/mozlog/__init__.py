# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

"""
Mozlog aims to standardize log handling and formatting within Mozilla.

It implements a JSON-based structured logging protocol with convenience
facilities for recording test results.

The old unstructured module is deprecated. It simply wraps Python's
logging_ module and adds a few convenience methods for logging test
results and events.
"""

import sys

from . import commandline, structuredlog, unstructured
from .proxy import get_proxy_logger
from .structuredlog import get_default_logger, set_default_logger

# Backwards compatibility shim for consumers that use mozlog.structured
structured = sys.modules[__name__]
sys.modules["{}.structured".format(__name__)] = structured

__all__ = [
    "commandline",
    "structuredlog",
    "unstructured",
    "get_default_logger",
    "set_default_logger",
    "get_proxy_logger",
    "structured",
]
