"""
Tests for `attr.filters`.
"""

from __future__ import absolute_import, division, print_function

import pytest

import attr

from attr import fields
from attr.filters import _split_what, exclude, include


@attr.s
class C(object):
    a = attr.ib()
    b = attr.ib()


class TestSplitWhat(object):
    """
    Tests for `_split_what`.
    """
    def test_splits(self):
        """
        Splits correctly.
        """
        assert (
            frozenset((int, str)),
            frozenset((fields(C).a,)),
        ) == _split_what((str, fields(C).a, int,))


class TestInclude(object):
    """
    Tests for `include`.
    """
    @pytest.mark.parametrize("incl,value", [
        ((int,), 42),
        ((str,), "hello"),
        ((str, fields(C).a), 42),
        ((str, fields(C).b), "hello"),
    ])
    def test_allow(self, incl, value):
        """
        Return True if a class or attribute is whitelisted.
        """
        i = include(*incl)
        assert i(fields(C).a, value) is True

    @pytest.mark.parametrize("incl,value", [
        ((str,), 42),
        ((int,), "hello"),
        ((str, fields(C).b), 42),
        ((int, fields(C).b), "hello"),
    ])
    def test_drop_class(self, incl, value):
        """
        Return False on non-whitelisted classes and attributes.
        """
        i = include(*incl)
        assert i(fields(C).a, value) is False


class TestExclude(object):
    """
    Tests for `exclude`.
    """
    @pytest.mark.parametrize("excl,value", [
        ((str,), 42),
        ((int,), "hello"),
        ((str, fields(C).b), 42),
        ((int, fields(C).b), "hello"),
    ])
    def test_allow(self, excl, value):
        """
        Return True if class or attribute is not blacklisted.
        """
        e = exclude(*excl)
        assert e(fields(C).a, value) is True

    @pytest.mark.parametrize("excl,value", [
        ((int,), 42),
        ((str,), "hello"),
        ((str, fields(C).a), 42),
        ((str, fields(C).b), "hello"),
    ])
    def test_drop_class(self, excl, value):
        """
        Return True on non-blacklisted classes and attributes.
        """
        e = exclude(*excl)
        assert e(fields(C).a, value) is False
