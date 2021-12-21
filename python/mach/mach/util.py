# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import absolute_import, unicode_literals

import os
import sys

from six import text_type


def setenv(key, value):
    """Compatibility shim to ensure the proper string type is used with
    os.environ for the version of Python being used.
    """
    encoding = "mbcs" if sys.platform == "win32" else "utf-8"

    if sys.version_info[0] == 2:
        if isinstance(key, text_type):
            key = key.encode(encoding)
        if isinstance(value, text_type):
            value = value.encode(encoding)
    else:
        if isinstance(key, bytes):
            key = key.decode(encoding)
        if isinstance(value, bytes):
            value = value.decode(encoding)

    os.environ[key] = value
