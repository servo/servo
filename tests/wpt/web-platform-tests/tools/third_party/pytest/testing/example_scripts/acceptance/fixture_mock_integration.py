# -*- coding: utf-8 -*-
"""Reproduces issue #3774"""

try:
    import mock
except ImportError:
    import unittest.mock as mock

import pytest

config = {"mykey": "ORIGINAL"}


@pytest.fixture(scope="function")
@mock.patch.dict(config, {"mykey": "MOCKED"})
def my_fixture():
    return config["mykey"]


def test_foobar(my_fixture):
    assert my_fixture == "MOCKED"
