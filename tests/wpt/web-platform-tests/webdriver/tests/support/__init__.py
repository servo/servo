import sys

from .merge_dictionaries import merge_dictionaries

platform_name = {
    # From Python version 3.3: On Linux, sys.platform doesn't contain the major version anymore.
    # It is always 'linux'. See
    # https://docs.python.org/3/library/sys.html#sys.platform
    "linux": "linux",
    "linux2": "linux",
    "win32": "windows",
    "cygwin": "windows",
    "darwin": "mac"
}.get(sys.platform)
