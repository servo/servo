import pytest
import sys

import webdriver


def product(a, b):
    return [(a, item) for item in b]


def flatten(l):
    return [item for x in l for item in x]

@pytest.fixture(scope="session")
def platform_name():
    return {
        "linux2": "linux",
        "win32": "windows",
        "cygwin": "windows",
        "darwin": "mac"
    }.get(sys.platform)

