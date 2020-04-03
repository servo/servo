import sys

from .merge_dictionaries import merge_dictionaries

platform_name = {
    "linux2": "linux",
    "win32": "windows",
    "cygwin": "windows",
    "darwin": "mac"
}.get(sys.platform)
