"""
Tests for `attr.converters`.
"""

from __future__ import absolute_import

import pytest

from attr.converters import optional


class TestOptional(object):
    """
    Tests for `optional`.
    """
    def test_success_with_type(self):
        """
        Wrapped converter is used as usual if value is not None.
        """
        c = optional(int)
        assert c("42") == 42

    def test_success_with_none(self):
        """
        Nothing happens if None.
        """
        c = optional(int)
        assert c(None) is None

    def test_fail(self):
        """
        Propagates the underlying conversion error when conversion fails.
        """
        c = optional(int)
        with pytest.raises(ValueError):
            c("not_an_int")
