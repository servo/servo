from __future__ import absolute_import, division, unicode_literals

import six
from mock import Mock

from . import support


def _createReprMock(r):
    """Creates a mock with a __repr__ returning r

    Also provides __str__ mock with default mock behaviour"""
    mock = Mock()
    mock.__repr__ = Mock()
    mock.__repr__.return_value = r
    mock.__str__ = Mock(wraps=mock.__str__)
    return mock


def test_errorMessage():
    # Create mock objects to take repr of
    input = _createReprMock("1")
    expected = _createReprMock("2")
    actual = _createReprMock("3")

    # Run the actual test
    r = support.errorMessage(input, expected, actual)

    # Assertions!
    if six.PY2:
        assert b"Input:\n1\nExpected:\n2\nRecieved\n3\n" == r
    else:
        assert six.PY3
        assert "Input:\n1\nExpected:\n2\nRecieved\n3\n" == r

    assert input.__repr__.call_count == 1
    assert expected.__repr__.call_count == 1
    assert actual.__repr__.call_count == 1
    assert not input.__str__.called
    assert not expected.__str__.called
    assert not actual.__str__.called
