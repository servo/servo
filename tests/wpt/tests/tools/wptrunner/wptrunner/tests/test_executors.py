# mypy: allow-untyped-defs

import pytest

from ..executors import base

@pytest.mark.parametrize("ranges_value, total_pages, expected", [
    ([], 3, {1, 2, 3}),
    ([[1, 2]], 3, {1, 2}),
    ([[1], [3, 4]], 5, {1, 3, 4}),
    ([[1],[3]], 5, {1, 3}),
    ([[2, None]], 5, {2, 3, 4, 5}),
    ([[None, 2]], 5, {1, 2}),
    ([[None, 2], [2, None]], 5, {1, 2, 3, 4, 5}),
    ([[1], [6, 7], [8]], 5, {1})])
def test_get_pages_valid(ranges_value, total_pages, expected):
    assert base.get_pages(ranges_value, total_pages) == expected
