# mypy: allow-untyped-defs
import pytest


def test_foo():
    assert True


@pytest.mark.parametrize("i", range(3))
def test_bar(i):
    assert True
