from __future__ import absolute_import, division, print_function

import sys

import pytest


@pytest.fixture(scope="session")
def C():
    """
    Return a simple but fully featured attrs class with an x and a y attribute.
    """
    import attr

    @attr.s
    class C(object):
        x = attr.ib()
        y = attr.ib()

    return C


collect_ignore = []
if sys.version_info[:2] < (3, 6):
    collect_ignore.extend([
        "tests/test_annotations.py",
        "tests/test_init_subclass.py",
    ])
