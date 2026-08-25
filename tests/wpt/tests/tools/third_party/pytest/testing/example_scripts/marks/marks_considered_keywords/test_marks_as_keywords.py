# mypy: allow-untyped-defs
import pytest


@pytest.mark.foo
def test_mark():
    pass
