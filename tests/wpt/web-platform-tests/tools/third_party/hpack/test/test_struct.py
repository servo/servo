# -*- coding: utf-8 -*-
"""
test_struct
~~~~~~~~~~~

Tests for the Header tuples.
"""
import pytest

from hpack.struct import HeaderTuple, NeverIndexedHeaderTuple


class TestHeaderTuple(object):
    def test_is_tuple(self):
        """
        HeaderTuple objects are tuples.
        """
        h = HeaderTuple('name', 'value')
        assert isinstance(h, tuple)

    def test_unpacks_properly(self):
        """
        HeaderTuple objects unpack like tuples.
        """
        h = HeaderTuple('name', 'value')
        k, v = h

        assert k == 'name'
        assert v == 'value'

    def test_header_tuples_are_indexable(self):
        """
        HeaderTuple objects can be indexed.
        """
        h = HeaderTuple('name', 'value')
        assert h.indexable

    def test_never_indexed_tuples_are_not_indexable(self):
        """
        NeverIndexedHeaderTuple objects cannot be indexed.
        """
        h = NeverIndexedHeaderTuple('name', 'value')
        assert not h.indexable

    @pytest.mark.parametrize('cls', (HeaderTuple, NeverIndexedHeaderTuple))
    def test_equal_to_tuples(self, cls):
        """
        HeaderTuples and NeverIndexedHeaderTuples are equal to equivalent
        tuples.
        """
        t1 = ('name', 'value')
        t2 = cls('name', 'value')

        assert t1 == t2
        assert t1 is not t2

    @pytest.mark.parametrize('cls', (HeaderTuple, NeverIndexedHeaderTuple))
    def test_equal_to_self(self, cls):
        """
        HeaderTuples and NeverIndexedHeaderTuples are always equal when
        compared to the same class.
        """
        t1 = cls('name', 'value')
        t2 = cls('name', 'value')

        assert t1 == t2
        assert t1 is not t2

    def test_equal_for_different_indexes(self):
        """
        HeaderTuples compare equal to equivalent NeverIndexedHeaderTuples.
        """
        t1 = HeaderTuple('name', 'value')
        t2 = NeverIndexedHeaderTuple('name', 'value')

        assert t1 == t2
        assert t1 is not t2
