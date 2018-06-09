from __future__ import absolute_import, division, unicode_literals

from html5lib.filters.optionaltags import Filter


def test_empty():
    assert list(Filter([])) == []
