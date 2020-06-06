import pytest

from ..executors import base

@pytest.mark.parametrize("ranges_value, total_pages, expected", [
    ("", 3, {1,2, 3}),
    ("1-2", 3, {1,2}),
    ("1-1,3-4", 5, {1,3,4}),
    ("1,3", 5, {1,3}),
    ("2-", 5, {2,3,4,5}),
    ("-2", 5, {1,2}),
    ("-2,2-", 5, {1,2,3,4,5}),
    ("1,6-7,8", 5, {1})])
def test_get_pages_valid(ranges_value, total_pages, expected):
    assert base.get_pages(ranges_value, total_pages) == expected


@pytest.mark.parametrize("ranges_value", ["a", "1-a", "1=2", "1-2:2-3"])
def test_get_pages_invalid(ranges_value):
    with pytest.raises(ValueError):
        assert base.get_pages(ranges_value, 1)
